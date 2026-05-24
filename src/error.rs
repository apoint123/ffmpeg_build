use std::ffi::CStr;
use thiserror::Error;

use crate::sys;

// https://github.com/FFmpeg/FFmpeg/blob/239f2c733de417201d7ad3b3b8b0d9b63285b2b1/libavutil/error.h#L86
const AV_ERROR_MAX_STRING_SIZE: usize = 64;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("End of file")]
    Eof,

    #[error("Resource temporarily unavailable")]
    Eagain,

    #[error("FFmpeg error {0}: {1}")]
    FFmpeg(i32, String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl AudioError {
    pub fn from_ffmpeg(code: i32) -> Self {
        match code {
            sys::AVERROR_EOF => AudioError::Eof,
            sys::AVERROR_EAGAIN => AudioError::Eagain,
            _ => {
                let mut buf = [0u8; AV_ERROR_MAX_STRING_SIZE];

                // SAFETY: buf is a valid 64-byte array that exists on the stack.
                // The passed pointer and length matches the array itself.
                unsafe {
                    sys::av_strerror(code, buf.as_mut_ptr() as *mut libc::c_char, buf.len());
                }

                let error_message = CStr::from_bytes_until_nul(&buf)
                    .map(|c_str| c_str.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| "Unknown FFmpeg error when parsing C string".to_string());

                AudioError::FFmpeg(code, error_message)
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, AudioError>;

/// A convenient macro used to quickly catch errors after calling to a FFmpeg C function
///
/// If the return value is >= 0, return that value directly; if < 0, automatically convert it to [`AudioError`] and throw it with a ?
#[macro_export]
macro_rules! fferr {
    ($expr:expr) => {{
        let ret = $expr;
        if ret < 0 {
            return Err($crate::error::AudioError::from_ffmpeg(ret));
        }
        ret
    }};
}
