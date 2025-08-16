#!/usr/bin/env bash

OS_TYPE="$(uname -s)"

case "$OS_TYPE" in
    Linux*)
        echo "Linux erkannt"
        ./linux_start   # hier dein Linux-Skript
        ;;
    Darwin*)
        echo "macOS erkannt"
        echo "macOS nicht supported (heheha)"
        # ./macos_script.sh   # hier dein macOS-Skript
        ;;
    CYGWIN*|MINGW*|MSYS*)
        echo "Windows erkannt"
        ./windows_start.sh   # Windows-Skript
        ;;
    *)
        echo "Unbekanntes System: $OS_TYPE"
        ;;
esac
