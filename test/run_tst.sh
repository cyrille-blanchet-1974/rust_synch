#/bin/sh
cd ..
cargo build
cp target/debug/synch test
cd test
./synch /conf:conftstux.txt /fic:run1.sh /verbose
pause
