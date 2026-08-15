use std::fmt;

use serde::{Serialize, de::DeserializeOwned};

pub const MAX_FRAME_BYTES: usize = 1024 * 1024;
pub const FRAME_PREFIX_BYTES: usize = size_of::<u32>();

pub fn encode_frame<T: DeserializeOwned + Serialize>(message: &T) -> Result<Vec<u8>, CodecError> {
    let payload = serde_json::to_vec(message).map_err(CodecError::Json)?;
    if payload.len() > MAX_FRAME_BYTES {
        return Err(CodecError::FrameTooLarge {
            actual: payload.len(),
            maximum: MAX_FRAME_BYTES,
        });
    }
    let length = u32::try_from(payload.len()).map_err(|_| CodecError::FrameTooLarge {
        actual: payload.len(),
        maximum: MAX_FRAME_BYTES,
    })?;
    serde_json::from_slice::<T>(&payload).map_err(CodecError::Json)?;
    let mut frame = Vec::with_capacity(FRAME_PREFIX_BYTES + payload.len());
    frame.extend_from_slice(&length.to_le_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

pub fn decode_payload<T: DeserializeOwned>(payload: &[u8]) -> Result<T, CodecError> {
    if payload.len() > MAX_FRAME_BYTES {
        return Err(CodecError::FrameTooLarge {
            actual: payload.len(),
            maximum: MAX_FRAME_BYTES,
        });
    }
    serde_json::from_slice(payload).map_err(CodecError::Json)
}

#[derive(Debug)]
pub enum CodecError {
    FrameTooLarge { actual: usize, maximum: usize },
    Json(serde_json::Error),
}

impl fmt::Display for CodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameTooLarge { actual, maximum } => {
                write!(
                    formatter,
                    "IPC frame is {actual} bytes; maximum is {maximum}"
                )
            }
            Self::Json(_) => formatter.write_str("IPC payload is not valid protocol JSON"),
        }
    }
}

impl std::error::Error for CodecError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::FrameTooLarge { .. } => None,
        }
    }
}
