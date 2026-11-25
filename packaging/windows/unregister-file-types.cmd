@echo off
rem Remove what register-file-types.cmd added.
reg delete "HKCU\Software\Classes\.wsbkz" /f >nul 2>&1
reg delete "HKCU\Software\Classes\WordsmithBook.Packed" /f >nul 2>&1
reg delete "HKCU\Software\Classes\Directory\shell\WordsmithBook" /f >nul 2>&1
echo Wordsmith Book's project types are removed.
