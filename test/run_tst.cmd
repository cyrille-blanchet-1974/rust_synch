::@echo off
cd ..
cargo build
copy target\debug\synch.exe test
cd test
synch /conf:conftst.txt /fic:run1.cmd /verbose
pause