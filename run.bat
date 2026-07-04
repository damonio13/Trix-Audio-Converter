@echo off
rem Trix Audio Converter — Desktop Launcher with Live Reload
title Trix Audio Converter
cd /d "%~dp0"

rem ── Encerrar instâncias anteriores ──────────────────────────────────────
taskkill /F /IM trix.exe >nul 2>&1

rem ── Iniciar servidor Vite em segundo plano ───────────────────────────────
echo [1/3] Iniciando servidor frontend (Vite)...
start "" /min cmd /c "npm run dev"

rem ── Aguardar a porta 8888 ficar disponível (até 30s) ────────────────────
echo [2/3] Aguardando Vite ficar pronto na porta 8888...
set /a tentativas=0
:aguarda_vite
    set /a tentativas+=1
    if %tentativas% gtr 60 (
        echo ERRO: Vite nao respondeu em 30 segundos. Verifique o npm.
        pause
        exit /b 1
    )
    powershell -NoProfile -Command "try { $r = Invoke-WebRequest -Uri 'http://localhost:8888' -UseBasicParsing -TimeoutSec 1 -ErrorAction Stop; exit 0 } catch { exit 1 }" >nul 2>&1
    if errorlevel 1 (
        ping -n 2 127.0.0.1 >nul
        goto aguarda_vite
    )
echo     Vite pronto!

rem ── Lançar executável ────────────────────────────────────────────────────
echo [3/3] Iniciando Trix Audio Converter...
if exist src-rs\target\release\trix.exe (
    src-rs\target\release\trix.exe --gui
) else (
    echo Executavel nao encontrado. Compilando - isso pode demorar...
    cargo build --release --manifest-path src-rs\Cargo.toml
    if errorlevel 1 (
        echo ERRO: Falha na compilacao.
        pause
        exit /b 1
    )
    src-rs\target\release\trix.exe --gui
)
