@echo off
setlocal
rem ============================================
rem  NPP-SIM — портативный запускатель
rem  Ищет Java: 1) папка "jre" рядом с файлом
rem             2) %JAVA_HOME%
rem             3) java/javaw в PATH
rem ============================================
cd /d "%~dp0"

if exist "%~dp0jre\bin\javaw.exe" (
  start "" "%~dp0jre\bin\javaw.exe" -jar "%~dp0NPPsim.jar"
  exit /b 0
)
if defined JAVA_HOME if exist "%JAVA_HOME%\bin\javaw.exe" (
  start "" "%JAVA_HOME%\bin\javaw.exe" -jar "%~dp0NPPsim.jar"
  exit /b 0
)
where javaw >nul 2>nul
if %errorlevel%==0 (
  start "" javaw -jar "%~dp0NPPsim.jar"
  exit /b 0
)
where java >nul 2>nul
if %errorlevel%==0 (
  java -jar "%~dp0NPPsim.jar"
  exit /b 0
)

echo.
echo [NPP-SIM] Java Runtime не найден.
echo Установите JRE 8 или новее: https://adoptium.net
echo или положите папку "jre" рядом с этим файлом (NPPsim.bat).
echo.
pause
