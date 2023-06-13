curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
export PATH="$HOME/.cargo/bin:$PATH"
rustup component add rustfmt
rustup component add clippy
rustup component add llvm-tools-preview
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl

chmod +x setup.sh
./setup.sh

cargo install diesel_cli --no-default-features --features "sqlite-bundled"
cp $HOME/.cargo/bin/diesel .