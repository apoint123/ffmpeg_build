### 这是什么？

一个用来测试在 Rust 上编译 FFmpeg C 代码的仓库

### 构建

```bash
git clone --recurse-submodules https://github.com/apoint123/ffmpeg-build.git

# 或者在克隆完成后 git submodule update --init --recursive

cargo build

cargo test -- --nocapture
```
