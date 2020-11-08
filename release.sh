#/bin/sh*
cargo build --release
find target/release -type f -perm /a+x -exec cp {} bin \;
echo "Press Any Key..."
read a