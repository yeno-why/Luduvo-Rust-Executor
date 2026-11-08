@echo off
setlocal
cargo build --release
if errorlevel 1 (
    echo Build FAILED
    endlocal
    exit /b 1
)
echo Build SUCCESS
endlocal
exit /b 0