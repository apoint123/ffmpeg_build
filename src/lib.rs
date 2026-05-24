pub mod error;
pub mod io;
pub mod sys;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_ffmpeg_version() {
        unsafe {
            let version_ptr = sys::av_version_info();

            let version_str = CStr::from_ptr(version_ptr)
                .to_str()
                .expect("Failed to parse version string");

            println!("✅ FFmpeg version: {version_str}");
        }
    }
}
