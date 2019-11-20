# rust_synch
util to synchronize folders<br>
<br>
use multiple thread (6) to treat<br>
<br>
-one thread read source folders (list of sources)<br>
-a second thread read destination folders (liste of destinations)<br>
they send what they have found in a Multiple Producer Single Consumer chanel<br>
-At the other side of the chanel a thisrd thread get the result and when it get a set of matching source/destination it send them in two chanel<br>
-At the end of these two chanel we have thread four and five<br>
The first search what is in source and not in destination and generate copies (recursive if it is a folder<br>
If the file/folder is in source AND destination it compare them (file/folder content in the case of a folder.... size and date in the case of a file) and copy it if different<br>
The second comparison thread search object in destination that do not exist anymore in source and generate a delete command (car be a recursive delete for folders)<br>
These two threads (4 and 5) send the commands they generate thru a MPSC chanel<br>
-At the end of this chanel the last thread (6) receive commands and write them to a script file<br>
<br>
<br>
What remain TODO<br>
-test<br>
-generate unix command optionnaly<br>
-...<br>
