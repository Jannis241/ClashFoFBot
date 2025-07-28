import json
import os
from ultralytics import YOLO
import argparse

data = []

def write_data(data):
    file_path = "Communication/data.json"
    os.makedirs(os.path.dirname(file_path), exist_ok=True)
    with open(file_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=4)
    print(f"Daten erfolgreich in {file_path} geschrieben.")

IMAGE_PATH = "Communication/screenshot.png"
DATA_YAML = "dataset/data.yaml"
BEST_MODEL_PATH = 'runs/train/exp/weights/best.pt'

def create_new_model(epochs: int = 50):
    print("Training new model..")
    model = YOLO('yolov8n.pt')
    model.train(data=DATA_YAML, epochs=epochs)
    print("Training abgeschlossen. Das beste Modell findest du unter 'runs/train/exp/weights/best.pt'")

def continue_training(model_path=BEST_MODEL_PATH, epochs: int = 1):
    if not os.path.exists(model_path):
        print(f"Modell {model_path} nicht gefunden. Erstelle neues Model und trainiere es.")
        create_new_model(epochs)
    print("Gefundenes Modell:", model_path)
    print("Setze Training fort..")
    model = YOLO(model_path)
    model.train(data=DATA_YAML, epochs=epochs)
    print("Weitertraining abgeschlossen.")

def get_prediction(image_path, model_path=BEST_MODEL_PATH):
    print("Prediction mit Modell:", model_path)
    print("Bild:", image_path)
    if not os.path.exists(model_path):
        print("Modellpfad nicht gefunden. Hast du ein Modell trainiert?")
        return
    model = YOLO(model_path)
    results = model.predict(source=image_path)
    return results

parser = argparse.ArgumentParser(description="Trainings- und Vorhersagemodus für YOLO Modell")
parser.add_argument('--predict', action='store_true', help='Mache eine Vorhersage')
parser.add_argument('--model-name', type=str, default=BEST_MODEL_PATH, help='Pfad zum Modell für Training oder Prediction')
parser.add_argument('--train-new', action='store_true', help='Starte ein neues Training')
parser.add_argument('--continue-train', action='store_true', help='Setze Training fort mit gegebenem Modell')
parser.add_argument('--epochs', type=int, default=None, help='Anzahl der Trainings-Epochen')

args = parser.parse_args()

# Bestimme Standard-Epochen
epochs_train_new = 50
epochs_continue = 1

if args.train_new:
    ep = args.epochs if args.epochs else epochs_train_new
    train_new_model(epochs=ep)

elif args.continue_train:
    ep = args.epochs if args.epochs else epochs_continue
    continue_training(model_path=args.model_name, epochs=ep)

if args.predict:
    pred = get_prediction(IMAGE_PATH, model_path=args.model_name)
    print("Predictionsergebnis:", pred)

write_data(data)

