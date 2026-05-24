use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

fn get_config_dir_name(target: &str) -> &'static str {
    if target.contains("windows") {
        if target.contains("x86_64") {
            return "build_out_windows_x86_64";
        }
    } else if target.contains("android") {
        if target.contains("aarch64") {
            return "build_out_android_arm64-v8a";
        }
        if target.contains("armv7") {
            return "build_out_android_armeabi-v7a";
        }
        if target.contains("i686") {
            return "build_out_android_x86";
        }
        if target.contains("x86_64") {
            return "build_out_android_x86_64";
        }
    } else if target.contains("ios") {
        if target.contains("aarch64") {
            return "build_out_ios_arm64";
        }
    } else if target.contains("darwin") || target.contains("macos") {
        if target.contains("x86_64") {
            return "build_out_macos_x86_64";
        }
    } else if target.contains("linux") {
        if target.contains("aarch64") {
            return "build_out_linux_arm64";
        }
        if target.contains("x86_64") {
            return "build_out_linux_x86_64";
        }
    }
    panic!("Unsupported or missing config for target: {target}");
}

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let ffmpeg_dir = PathBuf::from("ffmpeg");
    let config_dir_name = get_config_dir_name(&target);
    let config_dir = PathBuf::from("configs").join(config_dir_name);
    let log_path = config_dir.join("make_dryrun.log");

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed={}", log_path.display());

    let log_content = fs::read_to_string(&log_path).unwrap_or_else(|_| {
        panic!(
            "无法读取日志文件: {}. 请确认目标平台的日志存在",
            log_path.display()
        )
    });

    let mut c_files = HashSet::new();
    let mut defines = HashSet::new();
    let mut includes = HashSet::new();

    for line in log_content.lines() {
        if (line.contains("-c -o ") || line.contains("-c -Fo")) && line.contains(".c") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for &part in &parts {
                if part.starts_with("-D") && !part.starts_with("-DBUILDING_") {
                    let define = &part[2..];
                    if let Some((k, v)) = define.split_once('=') {
                        defines.insert((k.to_string(), Some(v.to_string())));
                    } else {
                        defines.insert((define.to_string(), None));
                    }
                } else if let Some(inc) = part.strip_prefix("-I") {
                    if inc == "." {
                        continue;
                    }

                    if let Some(idx) = inc.find("ffmpeg/") {
                        includes.insert(inc[idx..].to_string());
                    } else if let Some(idx) = inc.find("ffmpeg\\") {
                        includes.insert(inc[idx..].to_string());
                    }
                } else if part.ends_with(".c")
                    && let Some(idx) = part.find("libav").or_else(|| part.find("libsw"))
                {
                    c_files.insert(part[idx..].to_string());
                }
            }
        }
    }

    let mut build = cc::Build::new();
    build.include(&ffmpeg_dir);
    build.include(&config_dir);

    build.include(ffmpeg_dir.join("libavcodec"));
    build.include(ffmpeg_dir.join("libavformat"));
    build.include(ffmpeg_dir.join("libswresample"));

    for inc in &includes {
        build.include(inc);
    }

    for (k, v) in &defines {
        if let Some(val) = v {
            build.define(k, val.as_str());
        } else {
            build.define(k, None);
        }
    }

    for file in c_files {
        build.file(ffmpeg_dir.join(file));
    }

    if target.contains("windows") {
        build.flag("/utf-8");
    }

    build.compile("ffmpeg_audio");

    if target.contains("linux") || target.contains("android") || target.contains("darwin") {
        println!("cargo:rustc-link-lib=m");
        println!("cargo:rustc-link-lib=pthread");
    } else if target.contains("windows") {
        println!("cargo:rustc-link-lib=bcrypt");
    }

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", ffmpeg_dir.display()))
        .clang_arg(format!("-I{}", config_dir.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("av_.*")
        .allowlist_function("avformat_.*")
        .allowlist_function("avcodec_.*")
        .allowlist_function("avio_.*")
        .allowlist_function("swr_.*")
        .allowlist_type("AV.*")
        .allowlist_type("Swr.*")
        .allowlist_var("AV_.*")
        .allowlist_var("AVERROR_.*");

    for inc in &includes {
        builder = builder.clang_arg(format!("-I{}", inc));
    }

    for (k, v) in defines {
        if let Some(val) = v {
            builder = builder.clang_arg(format!("-D{}={}", k, val));
        } else {
            builder = builder.clang_arg(format!("-D{}", k));
        }
    }

    let bindings = builder.generate().expect("Unable to generate bindings");
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
