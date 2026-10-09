@echo off
title Apocalypse 3D Launcher
echo ===================================================
echo     APOCALYPSE 3D - Vulkan Engine FPS (Rust)
echo ===================================================
echo Lancement du moteur de jeu...
cd /d "%~dp0"
cargo run --release
pause
