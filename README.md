# rust_synch
util to synchronize folders

use multiple thread (6) to treat

-one thread read source folders (list of sources)
-a second thread read destination folders (liste of destinations)
they send what they have found in a Multiple Producer Single Consumer chanel
-At the other side of the chanel a thisrd thread get the result and when it get a set of matching source/destination it send them in two chanel
-At the end of these two chanel we have thread four and five
The first search what is in source and not in destination and generate copies (recursive if it is a folder
If the file/folder is in source AND destination it compare them (file/folder content in the case of a folder.... size and date in the case of a file) and copy it if different
The second comparison thread search object in destination that do not exist anymore in source and generate a delete command (car be a recursive delete for folders)
These two threads (4 and 5) send the commands they generate thru a MPSC chanel
-At the end of this chanel the last thread (6) receive commands and write them to a script file


What remain TODO
-test
-generate unix command optionnaly
-...
