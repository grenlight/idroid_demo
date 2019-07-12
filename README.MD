# Demo rendering error on iOS

## Run on macOS
>$ cargo run --example compute --features metal

## Compile iOS .a file
Set Cargo.toml crate-type value to ["staticlib"], then:
>$ cargo build --target aarch64-apple-ios --features metal --release
