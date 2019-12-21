#/bin/sh
cd ..
cargo build
cp target/debug/synch test
cd test
./synch /conf:conftstux.txt /fic:run1.sh /verbose
chmod +x run1.sh
echo launch ./run1.sh to operate synchronisation
