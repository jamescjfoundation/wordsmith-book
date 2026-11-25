@echo off
rem Register Wordsmith Book's project types for the current user: .wsbkz files, and .wsbk
rem project folders (open book.toml inside one, or the folder's .wsbk name, with the app).
set "APP=%~dp0wordsmith-book.exe"
reg add "HKCU\Software\Classes\.wsbkz" /ve /d "WordsmithBook.Packed" /f >nul
reg add "HKCU\Software\Classes\WordsmithBook.Packed" /ve /d "Wordsmith Book project (one file)" /f >nul
reg add "HKCU\Software\Classes\WordsmithBook.Packed\DefaultIcon" /ve /d "\"%APP%\",0" /f >nul
reg add "HKCU\Software\Classes\WordsmithBook.Packed\shell\open\command" /ve /d "\"%APP%\" \"%%1\"" /f >nul
reg add "HKCU\Software\Classes\Directory\shell\WordsmithBook" /ve /d "Open with Wordsmith Book" /f >nul
reg add "HKCU\Software\Classes\Directory\shell\WordsmithBook" /v "AppliesTo" /d "System.FileName:\"*.wsbk\"" /f >nul
reg add "HKCU\Software\Classes\Directory\shell\WordsmithBook\command" /ve /d "\"%APP%\" \"%%1\"" /f >nul
echo Wordsmith Book's project types are registered.
