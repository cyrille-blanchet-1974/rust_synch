#/bin/sh*
cargo build --release
find target/release  -maxdepth 0 -type f -perm /a+x -exec cp {} ../bin \;
echo "Press Any Key..."
read a