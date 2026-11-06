@echo off
setlocal
set "BUILD_PROFILE=debug"
set "BUILD_FLAGS="

if "%~1"=="" goto build
if /I "%~1"=="--release" goto release
if /I "%~1"=="-Release" goto release
if /I "%~1"=="release" goto release
echo Usage: build.cmd [--release]
exit /b 1

:release
set "BUILD_PROFILE=release"
set "BUILD_FLAGS=--release"

:build
pushd "%~dp0"
if errorlevel 1 exit /b 1
cargo build --lib --target-dir "%~dp0target" %BUILD_FLAGS%
if errorlevel 1 goto build_failed

copy /Y "%~dp0target\%BUILD_PROFILE%\test_dll.dll" "C:\Windows\Temp\test_dll.dll"
if errorlevel 1 goto copy_failed
echo DLL copied to C:\Windows\Temp\test_dll.dll
popd
exit /b 0

:build_failed
echo ERROR: Build failed. DLL was not copied.
popd
exit /b 1

:copy_failed
echo ERROR: Could not copy DLL to C:\Windows\Temp\test_dll.dll.
popd
exit /b 1
