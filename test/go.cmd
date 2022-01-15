@echo off
echo cleaning
if exist src rd /S /q src
if exist dst rd /S /q dst
if exist res rd /S /q res
echo create source folder
md src
md src\1
echo mdkfgjmlksdfjgmskdjfg > src\1.txt
echo mdkfgjmlksdfjgmskdjfg > src\1\2.txt
md src\2
md src\2\3
md dst
echo get app
copy ..\target\release\synch.exe .
md res
pause
echo ............................................
echo test1: call with parm
synch  /src:src /dst:dst /fic:res\1.cmd
echo result should contain 3 recursive copies
type res\1.cmd
pause

echo ............................................
echo test2: call with conf file
echo source=src>conf.fic
echo destination=dst>>conf.fic
synch /conf:conf.fic /fic:res\2.cmd
echo result should contain 3 copies
type res\2.cmd
pause

::synchronise
call res\2.cmd
::add a file
echo mdkfgjmlksdfjgmskdjfg > src\2.txt
echo ............................................
echo test3: add a file
synch /conf:conf.fic /fic:res\3.cmd
echo result should contain 1 copy
type res\3.cmd
pause

::synchronise
call res\3.cmd
::remove a file
del src\1\2.txt /q
echo ............................................
echo test4: remove a file
synch /conf:conf.fic /fic:res\4.cmd
echo result should contain 1 delete
type res\4.cmd
pause


::remove a folder
rd /s /q src\1
echo ............................................
echo test5: remove a folder
synch /conf:conf.fic /fic:res\5.cmd
echo result should contain 1 recurse delete
type res\5.cmd
pause


::synchronise
call res\5.cmd
::change date of a file 
echo mdkfgjmlksdfjgmskdjfg > src\2.txt
echo ............................................
echo test6: date change of a file (param ignore
synch /conf:conf.fic /fic:res\6.cmd /ignore_date_diff
echo result should contain no commands
type res\6.cmd
pause


echo ............................................
echo test7: date change of a file (param ignore removed
synch /conf:conf.fic /fic:res\7.cmd 
echo result should contain a copy
type res\7.cmd
pause


::change size 
echo mdkfgjmlkdfmglksmùdgfùmsdfkglssdfjgmskdjfg > src\2.txt
echo ............................................
echo test7: size change of a file (param ignore 
synch /conf:conf.fic /fic:res\8.cmd /ignore_date_diff
echo result should contain a copy
type res\8.cmd
pause
