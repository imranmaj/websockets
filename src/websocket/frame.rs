use std::sync::Mutex;

use once_cell::sync::OnceCell;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

use crate::error::WebSocketError;

const U16_MAX_MINUS_ONE: usize = (u16::MAX - 1) as usize;
const U64_MAX_MINUS_ONE: usize = (u64::MAX - 1) as usize;

static RNG_CELL: OnceCell<Mutex<ChaCha20Rng>> = OnceCell::new();

// https://tools.ietf.org/html/rfc6455#section-5.2
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
    pub(super) fn into_raw(self) -> Result<Vec<u8>, WebSocketError> {
        // calculate before moving payload out of self
        let is_control = self.is_control();
        let opcode = self.opcode();
        let fin = self.fin();

        let mut payload = match self {
            // https://tools.ietf.org/html/rfc6455#section-5.6
            Self::Text { payload, .. } => payload.into_bytes(),
            Self::Binary { payload, .. } => payload,
            // https://tools.ietf.org/html/rfc6455#section-5.5.1
            Self::Close { payload } => payload
                .map(|(status_code, reason)| {
                    let mut payload = status_code.to_be_bytes().to_vec();
                    payload.append(&mut reason.into_bytes());
                    payload
                })
                .unwrap_or(Vec::new()),
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
        let mut payload_len_bytes = payload.len().to_be_bytes().to_vec();
        let mut payload_len_data = match payload.len() {
            0..=125 => payload_len_bytes,
            126..=U16_MAX_MINUS_ONE => {
                let mut payload_len_data = vec![126];
                payload_len_data.append(&mut payload_len_bytes);
                payload_len_data
            }
            U16_MAX_MINUS_ONE..=U64_MAX_MINUS_ONE => {
                let mut payload_len_data = vec![127];
                payload_len_data.append(&mut payload_len_bytes);
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
        let mut masking_key = Vec::with_capacity(4);
        rng.fill_bytes(&mut masking_key);
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte = *byte ^ (masking_key[i % 4]);
        }

        raw_frame.append(&mut payload);
        Ok(raw_frame)
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

    fn from_raw() -> Result<Self, WebSocketError> {
        unimplemented!()
    }
}
