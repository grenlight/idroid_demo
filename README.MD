# Demo rendering error on iOS

## Run on macOS
Set Cargo.toml crate-type value to ["rlib"], then:
>$ cargo run --example demo --features metal

## Compile iOS .a file
Set Cargo.toml crate-type value to ["staticlib"], then:
>$ cargo build --target aarch64-apple-ios --features metal --release
