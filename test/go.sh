#/bin/sh
function pause {
    read -n 120 -p "Press 'Enter' to continue..." ; echo " "
}

echo cleaning
rm -rf  src
rm -rf  dst
rm -rf  res
echo create source folder
mkdir src
mkdir src/1
echo mdkfgjmlksdfjgmskdjfg > src/1.txt
echo mdkfgjmlksdfjgmskdjfg > src/1/2.txt
mkdir src/2
mkdir src/2/3
mkdir dst
echo get app
cp ../target/release/synch .
mkdir res
pause
echo ............................................
echo test1: call with parm
./synch  /src:src /dst:dst /fic:res/1.sh
echo result should contain 3 recursive copies
cat res/1.sh
pause

echo ............................................
echo test2: call with conf file
echo source=src>conf.fic
echo destination=dst>>conf.fic
./synch /conf:conf.fic /fic:res/2.sh
echo result should contain 3 copies
cat res/2.sh
pause

#synchronise
chmod +x res/2.sh
res/2.sh
#add a file
echo mdkfgjmlksdfjgmskdjfg > src/2.txt
echo ............................................
echo test3: add a file
./synch /conf:conf.fic /fic:res/3.sh
echo result should contain 1 copy
cat res/3.sh
pause

#synchronise
chmod +x res/3.sh
res/3.sh
#remove a file
rm src/1/2.txt 
echo ............................................
echo test4: remove a file
./synch /conf:conf.fic /fic:res/4.sh
echo result should contain 1 delete
cat res/4.sh
pause


#remove a folder
rm -rf src/1
echo ............................................
echo test5: remove a folder
./synch /conf:conf.fic /fic:res/5.sh
echo result should contain 1 recurse delete
cat res/5.sh
pause


#synchronise
chmod +x res/5.sh
res/5.sh
#change date of a file 
echo mdkfgjmlksdfjgmskdjfg > src/2.txt
echo ............................................
echo test6: date change of a file with param ignore
./synch /conf:conf.fic /fic:res/6.sh /ignore_date_diff
echo result should contain no commands
cat res/6.sh
pause


echo ............................................
echo test7: date change of a file without param ignore
./synch /conf:conf.fic /fic:res/7.sh 
echo result should contain a copy
cat res/7.sh
pause


#change size 
echo mdkfgjmlkdfmglksmùdgfùmsdfkglssdfjgmskdjfg > src/2.txt
echo ............................................
echo test7: size change of a file with param ignore 
./synch /conf:conf.fic /fic:res/8.sh /ignore_date_diff
echo result should contain a copy
cat res/8.sh
pause
