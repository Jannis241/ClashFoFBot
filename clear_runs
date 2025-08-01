#!/bin/bash

# Stelle sicher, dass das Script im richtigen Verzeichnis ausgeführt wird
if [ ! -d "runs/detect" ]; then
    echo "Ordner runs/detect nicht gefunden. Skript wird abgebrochen."
    exit 1
fi

echo "Starte das Löschen von zugehörigen Stats-Ordnern..."

# Durchlaufe alle Unterordner in runs/detect (also alle Modelle)
for model_dir in runs/detect/*/; do
    model_name=$(basename "$model_dir")
    stats_path="Stats/$model_name"

    if [ -d "$stats_path" ]; then
        echo "Lösche zugehörigen Stats-Ordner: $stats_path"
        rm -rf "$stats_path"
    else
        echo "Kein Stats-Ordner gefunden für: $model_name"
    fi
done

# Lösche den gesamten runs-Ordner
echo "Lösche kompletten runs-Ordner..."
rm -rf runs

echo "Bereinigung abgeschlossen ✅"

