@echo off
chcp 65001 >nul
title QQ 智慧助理

echo ========================================
echo   QQ 智慧助理 - 启动脚本
echo ========================================
echo.

:: Set project root
set PROJECT_ROOT=D:\桌面\编程作品\Sandy ONE

:: Check if NapCatQQ is running (port 4447)
echo [1/3] 检查 NapCatQQ 连接状态...
netstat -an 2>nul | findstr "127.0.0.1:4447" >nul
if %errorlevel% equ 0 (
    echo   ✓ NapCatQQ WebSocket 端口 4447 已就绪
) else (
    echo   ! NapCatQQ WebSocket 端口 4447 未检测到
    echo   ! 请确保 NapCatQQ 已启动且 WebSocket 服务配置正确
    echo   ! 按任意键继续，或关闭窗口退出
    pause >nul
)

:: Check port 4444 (HTTP API)
netstat -an 2>nul | findstr "127.0.0.1:4444" >nul
if %errorlevel% equ 0 (
    echo   ✓ NapCatQQ HTTP 端口 4444 已就绪
) else (
    echo   ! NapCatQQ HTTP 端口 4444 未检测到
)
echo.

:: Build and run
echo [2/3] 编译 QQ 智慧助理...
cd /d "%PROJECT_ROOT%\qq-assistant"
cargo build --release 2>&1
if %errorlevel% neq 0 (
    echo   ✗ 编译失败，请查看上方错误信息
    pause
    exit /b 1
)
echo   ✓ 编译成功
echo.

echo [3/3] 启动 QQ 智慧助理...
echo.
echo 访问面板: http://127.0.0.1:5050
echo 按 Ctrl+C 停止
echo ========================================
echo.

cargo run --release
if %errorlevel% neq 0 (
    echo.
    echo 程序异常退出，按任意键关闭
    pause
)
