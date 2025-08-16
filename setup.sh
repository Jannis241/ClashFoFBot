#!/usr/bin/env bash

OS_TYPE="$(uname -s)"

case "$OS_TYPE" in
    Linux*)
        echo "Linux erkannt"
        scripts/linux_setup   # hier dein Linux-Skript
        ;;
    Darwin*)
        echo "macOS erkannt"
        echo "macOS nicht supported (heheha)"
        ;;
    CYGWIN*|MINGW*|MSYS*)
        echo "Windows erkannt"
        scripts/windows_setup.sh   # Windows-Skript
        ;;
    *)
        echo "Unbekanntes System: $OS_TYPE"
        ;;
esac
