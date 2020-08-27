use std::convert::TryInto;
use std::sync::Mutex;

use once_cell::sync::OnceCell;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::{FrameType, WebSocket};
use crate::error::WebSocketError;

const U16_MAX_MINUS_ONE: usize = (u16::MAX - 1) as usize;
const U64_MAX_MINUS_ONE: usize = (u64::MAX - 1) as usize;

static RNG_CELL: OnceCell<Mutex<ChaCha20Rng>> = OnceCell::new();

// https://tools.ietf.org/html/rfc6455#section-5.2
#[derive(Debug)]
pub enum Frame {
    Text {
        payload: String,
        continuation: bool,
        fin: bool,
    },
    Binary {
        payload: Vec<u8>,
        continuation: bool,
        fin: bool,
    },
    Close {
        payload: Option<(u16, String)>,
    },
    Ping {
        payload: Option<Vec<u8>>,
    },
    Pong {
        payload: Option<Vec<u8>>,
    },
}

impl Frame {
    pub(super) async fn send(self, ws: &mut WebSocket) -> Result<(), WebSocketError> {
        // calculate before moving payload out of self
        let is_control = self.is_control();
        let opcode = self.opcode();
        let fin = self.fin();

        let mut payload = match self {
            // https://tools.ietf.org/html/rfc6455#section-5.6
            Self::Text { payload, .. } => payload.into_bytes(),
            Self::Binary { payload, .. } => payload,
            // https://tools.ietf.org/html/rfc6455#section-5.5.1
            Self::Close {
                payload: Some((status_code, reason)),
            } => {
                let mut payload = status_code.to_be_bytes().to_vec();
                payload.append(&mut reason.into_bytes());
                payload
            }
            Self::Close { payload: None } => Vec::new(),
            // https://tools.ietf.org/html/rfc6455#section-5.5.2
            Self::Ping { payload } => payload.unwrap_or(Vec::new()),
            // https://tools.ietf.org/html/rfc6455#section-5.5.3
            Self::Pong { payload } => payload.unwrap_or(Vec::new()),
        };
        // control frame cannot be longer than 125 bytes: https://tools.ietf.org/html/rfc6455#section-5.5
        if is_control && payload.len() > 125 {
            return Err(WebSocketError::ControlFrameTooLargeError);
        }

        // set payload len: https://tools.ietf.org/html/rfc6455#section-5.2
        let mut raw_frame = Vec::with_capacity(payload.len() + 14);
        raw_frame.push(opcode + fin);
        let mut payload_len = payload.len().to_be_bytes().to_vec();
        let mut payload_len_data = match payload.len() {
            0..=125 => payload_len,
            126..=U16_MAX_MINUS_ONE => {
                let mut payload_len_data = vec![126];
                payload_len_data.append(&mut payload_len);
                payload_len_data
            }
            U16_MAX_MINUS_ONE..=U64_MAX_MINUS_ONE => {
                let mut payload_len_data = vec![127];
                payload_len_data.append(&mut payload_len);
                payload_len_data
            }
            _ => return Err(WebSocketError::PayloadTooLargeError),
        };
        payload_len_data[0] += 0b10000000; // set masking bit: https://tools.ietf.org/html/rfc6455#section-5.3
        raw_frame.append(&mut payload_len_data);

        // payload masking: https://tools.ietf.org/html/rfc6455#section-5.3
        let mut rng = RNG_CELL
            .get_or_init(|| Mutex::new(ChaCha20Rng::from_entropy()))
            .lock()
            .expect("rng mutex poisoned");
        let mut masking_key = vec![0; 4];
        rng.fill_bytes(&mut masking_key);
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte = *byte ^ (masking_key[i % 4]);
        }
        raw_frame.append(&mut masking_key);

        raw_frame.append(&mut payload);
        ws.stream
            .write_all(&raw_frame)
            .await
            .map_err(|e| WebSocketError::WriteError(e))?;
        ws.stream
            .flush()
            .await
            .map_err(|e| WebSocketError::WriteError(e))?;
        Ok(())
    }

    fn is_control(&self) -> bool {
        // control frames: https://tools.ietf.org/html/rfc6455#section-5.5
        match self {
            Self::Text { .. } => false,
            Self::Binary { .. } => false,
            Self::Close { .. } => true,
            Self::Ping { .. } => true,
            Self::Pong { .. } => true,
        }
    }

    fn opcode(&self) -> u8 {
        // opcodes: https://tools.ietf.org/html/rfc6455#section-5.2
        match self {
            Self::Text { continuation, .. } => {
                if *continuation {
                    0x0
                } else {
                    0x1
                }
            }
            Self::Binary { continuation, .. } => {
                if *continuation {
                    0x0
                } else {
                    0x2
                }
            }
            Self::Close { .. } => 0x8,
            Self::Ping { .. } => 0x9,
            Self::Pong { .. } => 0xA,
        }
    }

    fn fin(&self) -> u8 {
        // fin bit: https://tools.ietf.org/html/rfc6455#section-5.2
        match self {
            Self::Text { fin, .. } => (*fin as u8) << 7,
            Self::Binary { fin, .. } => (*fin as u8) << 7,
            Self::Close { .. } => 0,
            Self::Ping { .. } => 0,
            Self::Pong { .. } => 0,
        }
    }

    pub(super) async fn read_from_websocket(ws: &mut WebSocket) -> Result<Self, WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.2
        let fin_and_opcode = ws
            .stream
            .read_u8()
            .await
            .map_err(|e| WebSocketError::ReadError(e))?;
        let fin: bool = fin_and_opcode & 0b10000000_u8 != 0;
        let opcode = fin_and_opcode & 0b00001111_u8;

        let mask_and_payload_len_first_byte = ws
            .stream
            .read_u8()
            .await
            .map_err(|e| WebSocketError::ReadError(e))?;
        let masked = mask_and_payload_len_first_byte & 0b10000000_u8 != 0;
        if masked {
            // server to client frames should not be masked
            return Err(WebSocketError::ReceivedMaskedFrameError);
        }
        let payload_len_first_byte = mask_and_payload_len_first_byte & 0b01111111_u8;
        let payload_len = match payload_len_first_byte {
            0..=125 => payload_len_first_byte as usize,
            126 => ws
                .stream
                .read_u16()
                .await
                .map_err(|e| WebSocketError::ReadError(e))? as usize,
            127 => ws
                .stream
                .read_u64()
                .await
                .map_err(|e| WebSocketError::ReadError(e))? as usize,
            _ => unreachable!(),
        };

        let mut payload = vec![0; payload_len];
        ws.stream
            .read_exact(&mut payload)
            .await
            .map_err(|e| WebSocketError::ReadError(e))?;

        match opcode {
            0x0 => match ws.last_frame_type {
                FrameType::Text => Ok(Self::Text {
                    payload: String::from_utf8(payload)
                        .map_err(|_e| WebSocketError::InvalidFrameError)?,
                    continuation: true,
                    fin,
                }),
                FrameType::Binary => Ok(Self::Binary {
                    payload,
                    continuation: true,
                    fin,
                }),
                FrameType::Control => Err(WebSocketError::InvalidFrameError),
            },
            0x1 => Ok(Self::Text {
                payload: String::from_utf8(payload)
                    .map_err(|_e| WebSocketError::InvalidFrameError)?,
                continuation: false,
                fin,
            }),
            0x2 => Ok(Self::Binary {
                payload,
                continuation: false,
                fin,
            }),
            // reserved range
            0x3..=0x7 => Err(WebSocketError::InvalidFrameError),
            0x8 if payload.len() == 0 => Ok(Self::Close { payload: None }),
            // if there is a payload it must have a u16 status code
            0x8 if payload_len < 2 => Err(WebSocketError::InvalidFrameError),
            0x8 => {
                let (status_code, reason) = payload.split_at(2);
                let status_code = u16::from_be_bytes(
                    status_code
                        .try_into()
                        .map_err(|_e| WebSocketError::InvalidFrameError)?,
                );
                Ok(Self::Close {
                    payload: Some((
                        status_code,
                        String::from_utf8(reason.to_vec())
                            .map_err(|_e| WebSocketError::InvalidFrameError)?,
                    )),
                })
            }
            0x9 if payload_len == 0 => Ok(Self::Ping { payload: None }),
            0x9 => Ok(Self::Ping {
                payload: Some(payload),
            }),
            0xA if payload_len == 0 => Ok(Self::Pong { payload: None }),
            0xA => Ok(Self::Pong {
                payload: Some(payload),
            }),
            // reserved range
            0xB..=0xFF => Err(WebSocketError::InvalidFrameError),
        }
    }
}
