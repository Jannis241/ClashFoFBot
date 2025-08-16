#!/usr/bin/env bash


# 1. Venv-Ordner definieren
VENV_DIR="venv"

# 2. Venv erstellen (wenn noch nicht vorhanden)
if [ ! -d "$VENV_DIR" ]; then
    echo "Erstelle virtuelles Environment im Ordner '$VENV_DIR'..."
    python -m venv $VENV_DIR
else
    echo "Virtuelles Environment existiert bereits."
fi

# 3. Venv aktivieren (für Bash und Zsh)
venv\\Scripts\\activate.bat

# 4. Pip aktualisieren (optional, aber empfohlen)
pip install --upgrade pip


# 4.5 torch neu installieren für gpu (am ende des linkes cuda version hinzufügen (12.6 = 126 | 12.8 = 128 ...)
pip uninstall torch torchvision torchaudio -y
# alt geharadcodet pip3 install torch torchvision --index-url https://download.pytorch.org/whl/cu128
python pytorchgpuinstaller

# 5. Abhängigkeiten installieren
if [ -f "requirements.txt" ]; then
    echo "Installiere Pakete aus requirements.txt..."
    pip install -r requirements.txt
else
    echo "requirements.txt nicht gefunden!"
fi

pip install -r requirements.txt

# Todo: diese python ding da für nummern erkennen hier downloaden von git oder so

echo "Setup abgeschlossen"
read -n 1 -s
