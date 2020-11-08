#/bin/sh
cargo fmt
cargo clippy
cargo build 
echo "Press Any Key..."
read a