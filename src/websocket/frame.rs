use std::convert::TryInto;

use rand::RngCore;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::{FrameType, WebSocket};
use crate::error::WebSocketError;

const U16_MAX_MINUS_ONE: usize = (u16::MAX - 1) as usize;
const U16_MAX: usize = u16::MAX as usize;
const U64_MAX_MINUS_ONE: usize = (u64::MAX - 1) as usize;

// https://tools.ietf.org/html/rfc6455#section-5.2
/// Data which is sent and received through the WebSocket connection.
/// 
/// # Sending
/// 
/// To send a [`Frame`], you can construct it normally and use the [`WebSocket::send()`] method, 
/// or use the convenience methods for each frame type 
/// ([`send_text()`](WebSocket::send_text()), [`send_binary()`](WebSocket::send_binary()), 
/// [`close()`](WebSocket::close()), [`send_ping()`](WebSocket::send_ping()), 
/// and [`send_pong()`](WebSocket::send_pong())).
/// 
/// # Receiving
/// 
/// [`Frame`]s can be received through the [`WebSocket::receive()`] method.
/// To extract the underlying data from a received `Frame`, 
/// you can use the convenience methods [`as_text()`](Frame::as_text()),
/// [`as_binary()`](Frame::as_binary()), [`as_close()`](Frame::as_close()),
/// [`as_ping()`](Frame::as_ping()), and [`as_pong()`](Frame::as_pong()).
/// (and their `mut` counterparts), or simply `match`.
#[derive(Debug)]
pub enum Frame {
    /// A Text frame
    Text {
        /// The payload for the Text frame
        payload: String,
        /// Whether the Text frame is a continuation frame in the message
        continuation: bool,
        /// Whether the Text frame is the final frame in the message
        fin: bool,
    },
    /// A Binary frame
    Binary {
        /// The payload for the Binary frame
        payload: Vec<u8>,
        /// Whether the Binary frame is a continuation frame in the message
        continuation: bool,
        /// Whether the Binary frame is the final frame in the message
        fin: bool,
    },
    /// A Close frame
    Close {
        /// The payload for the Close frame
        payload: Option<(u16, String)>,
    },
    /// A Ping frame
    Ping {
        /// The payload for the Ping frame
        payload: Option<Vec<u8>>,
    },
    /// A Pong frame
    Pong {
        /// The payload for the Pong frame
        payload: Option<Vec<u8>>,
    },
}

impl Frame {
    /// Returns whether the frame is a Text frame.
    pub fn is_text(&self) -> bool {
        self.as_text().is_some()
    }

    /// Attempts to interpret the frame as a Text frame,
    /// returning a reference to the underlying data if it is, 
    /// and None otherwise.
    pub fn as_text(&self) -> Option<(&String, &bool, &bool)> {
        match self {
            Self::Text {
                payload,
                continuation,
                fin,
            } => Some((payload, continuation, fin)),
            _ => None,
        }
    }
    /// Attempts to interpret the frame as a Text frame,
    /// returning a mutable reference to the underlying data if it is, 
    /// and None otherwise.
    pub fn as_text_mut(&mut self) -> Option<(&mut String, &mut bool, &mut bool)> {
        match self {
            Self::Text {
                payload,
                continuation,
                fin,
            } => Some((payload, continuation, fin)),
            _ => None,
        }
    }

    /// Returns whether the frame is a Binary frame.
    pub fn is_binary(&self) -> bool {
        self.as_binary().is_some()
    }

    /// Attempts to interpret the frame as a Binary frame,
    /// returning a reference to the underlying data if it is, 
    /// and None otherwise.
    pub fn as_binary(&self) -> Option<(&Vec<u8>, &bool, &bool)> {
        match self {
            Self::Binary {
                payload,
                continuation,
                fin,
            } => Some((payload, continuation, fin)),
            _ => None,
        }
    }

    /// Attempts to interpret the frame as a Binary frame,
    /// returning a mutable reference to the underlying data if it is, 
    /// and None otherwise.
    pub fn as_binary_mut(&mut self) -> Option<(&mut Vec<u8>, &mut bool, &mut bool)> {
        match self {
            Self::Binary {
                payload,
                continuation,
                fin,
            } => Some((payload, continuation, fin)),
            _ => None,
        }
    }

    /// Returns whether the frame is a Close frame.
    pub fn is_close(&self) -> bool {
        self.as_close().is_some()
    }

    /// Attempts to interpret the frame as a Close frame,
    /// returning a reference to the underlying data if it is, 
    /// and None otherwise.
    pub fn as_close(&self) -> Option<&(u16, String)> {
        match self {
            Self::Close { payload } => payload.as_ref(),
            _ => None,
        }
    }

    /// Attempts to interpret the frame as a Close frame,
    /// returning a mutable reference to the underlying data if it is, 
    /// and None otherwise.
    pub fn as_close_mut(&mut self) -> Option<&mut (u16, String)> {
        match self {
            Self::Close { payload } => payload.as_mut(),
            _ => None,
        }
    }

    /// Returns whether the frame is a Ping frame.
    pub fn is_ping(&self) -> bool {
        self.as_ping().is_some()
    }

    /// Attempts to interpret the frame as a Ping frame,
    /// returning a reference to the underlying data if it is, 
    /// and None otherwise.
    pub fn as_ping(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Ping { payload } => payload.as_ref(),
            _ => None,
        }
    }

    /// Attempts to interpret the frame as a Ping frame,
    /// returning a mutable reference to the underlying data if it is, 
    /// and None otherwise.
    pub fn as_ping_mut(&mut self) -> Option<&mut Vec<u8>> {
        match self {
            Self::Ping { payload } => payload.as_mut(),
            _ => None,
        }
    }

    /// Returns whether the frame is a Pong frame.
    pub fn is_pong(&self) -> bool {
        self.as_pong().is_some()
    }

    /// Attempts to interpret the frame as a Pong frame,
    /// returning a reference to the underlying data if it is, 
    /// and None otherwise.
    pub fn as_pong(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Pong { payload } => payload.as_ref(),
            _ => None,
        }
    }

    /// Attempts to interpret the frame as a Pong frame,
    /// returning a mutable reference to the underlying data if it is, 
    /// and None otherwise.
    pub fn as_pong_mut(&mut self) -> Option<&mut Vec<u8>> {
        match self {
            Self::Pong { payload } => payload.as_mut(),
            _ => None,
        }
    }

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
        let mut payload_len_data = match payload.len() {
            0..=125 => (payload.len() as u8).to_be_bytes().to_vec(),
            126..=U16_MAX_MINUS_ONE => {
                let mut payload_len_data = vec![126];
                payload_len_data.extend_from_slice(&(payload.len() as u16).to_be_bytes());
                payload_len_data
            }
            U16_MAX..=U64_MAX_MINUS_ONE => {
                let mut payload_len_data = vec![127];
                payload_len_data.extend_from_slice(&(payload.len() as u64).to_be_bytes());
                payload_len_data
            }
            _ => return Err(WebSocketError::PayloadTooLargeError),
        };
        payload_len_data[0] += 0b10000000; // set masking bit: https://tools.ietf.org/html/rfc6455#section-5.3
        raw_frame.append(&mut payload_len_data);

        // payload masking: https://tools.ietf.org/html/rfc6455#section-5.3
        let mut masking_key = vec![0; 4];
        ws.rng.fill_bytes(&mut masking_key);
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
            Self::Close { .. } => 0b10000000,
            Self::Ping { .. } => 0b10000000,
            Self::Pong { .. } => 0b10000000,
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
            0x8 if payload_len == 0 => Ok(Self::Close { payload: None }),
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
