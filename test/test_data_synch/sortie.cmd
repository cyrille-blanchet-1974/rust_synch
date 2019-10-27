@echo off
XCOPY "F:\dev\rust\test_data_synch\src\source_seulement\*.*" "F:\dev\rust\test_data_synch\dst" /E /I /H /Y /K /R 
XCOPY "F:\dev\rust\test_data_synch\src\des deux cotés\des deux cotés\source_seulement\*.*" "F:\dev\rust\test_data_synch\dst\des deux cotés\des deux cotés" /E /I /H /Y /K /R 
XCOPY "F:\dev\rust\test_data_synch\src\des deux cotés\des deux cotés\des deux cotés\différent.txt" "F:\dev\rust\test_data_synch\dst\des deux cotés\des deux cotés\des deux cotés" /H /Y /K /R 
XCOPY "F:\dev\rust\test_data_synch\src\des deux cotés\des deux cotés\des deux cotés\source_seulement.txt" "F:\dev\rust\test_data_synch\dst\des deux cotés\des deux cotés\des deux cotés" /H /Y /K /R 
XCOPY "F:\dev\rust\test_data_synch\src\des deux cotés\des deux cotés\source_seulement.txt" "F:\dev\rust\test_data_synch\dst\des deux cotés\des deux cotés" /H /Y /K /R 
XCOPY "F:\dev\rust\test_data_synch\src\des deux cotés\des deux cotés\différent.txt" "F:\dev\rust\test_data_synch\dst\des deux cotés\des deux cotés" /H /Y /K /R 
XCOPY "F:\dev\rust\test_data_synch\src\des deux cotés\source_seulement\*.*" "F:\dev\rust\test_data_synch\dst\des deux cotés" /E /I /H /Y /K /R 
XCOPY "F:\dev\rust\test_data_synch\src\des deux cotés\différent.txt" "F:\dev\rust\test_data_synch\dst\des deux cotés" /H /Y /K /R 
XCOPY "F:\dev\rust\test_data_synch\src\des deux cotés\source_seulement.txt" "F:\dev\rust\test_data_synch\dst\des deux cotés" /H /Y /K /R 
XCOPY "F:\dev\rust\test_data_synch\src\source_seulement.txt" "F:\dev\rust\test_data_synch\dst" /H /Y /K /R 
XCOPY "F:\dev\rust\test_data_synch\src\différent.txt" "F:\dev\rust\test_data_synch\dst" /H /Y /K /R 
RD /S /Q "F:\dev\rust\test_data_synch\dst\destination seulement"
Echo "F:\\dev\\rust\\test_data_synch\\dst\\gros destination seulement 2" Contains 0 folders and 101  files.
Echo Please confirm deletation
Echo Y to Delete
Echo N to keep
choice /C YN
if '%ERRORLEVE%'=='1' RD /S /Q "F:\dev\rust\test_data_synch\dst\gros destination seulement 2"
RD /S /Q "F:\dev\rust\test_data_synch\dst\des deux cotés\des deux cotés\destination seulement"
DEL "F:\dev\rust\test_data_synch\dst\des deux cotés\des deux cotés\des deux cotés\destination seulement.txt" /F /A 
DEL "F:\dev\rust\test_data_synch\dst\des deux cotés\des deux cotés\destination seulement.txt" /F /A 
RD /S /Q "F:\dev\rust\test_data_synch\dst\des deux cotés\destination seulement"
DEL "F:\dev\rust\test_data_synch\dst\des deux cotés\destination seulement.txt" /F /A 
Echo "F:\\dev\\rust\\test_data_synch\\dst\\gros destination seulement" Contains 11 folders and 0  files.
Echo Please confirm deletation
Echo Y to Delete
Echo N to keep
choice /C YN
if '%ERRORLEVE%'=='1' RD /S /Q "F:\dev\rust\test_data_synch\dst\gros destination seulement"
DEL "F:\dev\rust\test_data_synch\dst\destination seulement.txt" /F /A 
