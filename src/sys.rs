#![allow(nonstandard_style)]
#![allow(unnecessary_transmutes)]
#![allow(dead_code)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Returns a negative error code from a POSIX error code, to return from library functions.
pub const fn AVERROR(e: i32) -> i32 {
    -e
}

pub const fn MKTAG(a: u8, b: u8, c: u8, d: u8) -> i32 {
    (a as i32) | ((b as i32) << 8) | ((c as i32) << 16) | ((d as i32) << 24)
}

/// End of file
pub const AVERROR_EOF: i32 = -MKTAG(b'E', b'O', b'F', b' ');

/// Resource temporarily unavailable
pub const AVERROR_EAGAIN: i32 = AVERROR(libc::EAGAIN);

/// Not enough space
pub const AVERROR_ENOMEM: i32 = AVERROR(libc::ENOMEM);

/// Decoder not found
pub const AVERROR_DECODER_NOT_FOUND: i32 = -MKTAG(0xF8, b'D', b'E', b'C');

/// Passing this as the "whence" parameter to a seek function causes it to
/// return the filesize without seeking anywhere. Supporting this is optional.
/// If it is not supported then the seek function will return <0.
pub const AVSEEK_SIZE: i32 = 0x10000;

/// OR'ing this flag into the "whence" parameter to a seek function causes it to
/// seek by any means (like reopening and linear reading) or other normally unreasonable
/// means that can be extremely slow.
/// This is the default and therefore ignored by the seek code since 2010.
pub const AVSEEK_FORCE: i32 = 0x20000;
