import json
import os
from ultralytics import YOLO
import argparse


def write_data(data):
    file_path = "Communication/data.json"
    print("Writing data..")
    os.makedirs(os.path.dirname(file_path), exist_ok=True)
    with open(file_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=4)
    print(f"Daten erfolgreich in {file_path} geschrieben.")

IMAGE_PATH = "Communication/screenshot.png"
DATA_YAML = "dataset/data.yaml"
BEST_MODEL_PATH = 'runs/detect/train/weights/best.pt'

def create_new_model(epochs: int = 50):
    print("Training new model..")
    model = YOLO('yolov8n.pt')
    model.train(data=DATA_YAML, epochs=epochs)
    print("Training abgeschlossen. Das beste Modell findest du unter 'runs/train/exp/weights/best.pt'")

def continue_training(model_path=BEST_MODEL_PATH, epochs: int = 50):
    if not os.path.exists(model_path):
        print(f"Modell {model_path} nicht gefunden. Erstelle neues Model und trainiere es.")
        create_new_model(epochs)
    print("Gefundenes Modell:", model_path)
    print("Setze Training fort..")
    model = YOLO(model_path)
    model.train(data=DATA_YAML, epochs=epochs)
    print("Weitertraining abgeschlossen.")

def write_prediction_to_json(image_path, model_path=BEST_MODEL_PATH):
    print("Prediction mit Modell:", model_path)
    print("Image path used for prediction:", image_path)
    if not os.path.exists(model_path):
        print("Modellpfad nicht gefunden. Hast du ein Modell trainiert?")
        return

    # model = YOLO(model_path)
    model = YOLO('yolov8n.pt')

    results = model.predict(source=image_path)[0]
    class_names = model.names  # z. B. {0: "cannon", 1: "elixir", ...}

    output = []
    for box in results.boxes:
        cls_id = int(box.cls[0].item())              # class index (int)
        class_name = class_names[cls_id]             # class name (string)
        conf = float(box.conf[0].item())             # confidence score
        xyxy = box.xyxy[0].tolist()                  # bounding box [x1, y1, x2, y2]

        output.append({
            "class_id": cls_id,
            "class_name": class_name,
            "confidence": conf,
            "bounding_box": (xyxy[0], xyxy[1], xyxy[2], xyxy[3])
        })
    print("writing to json: ", output)
    write_data(output)

parser = argparse.ArgumentParser(description="Trainings- und Vorhersagemodus für YOLO Modell")
parser.add_argument('--predict', action='store_true', help='Mache eine Vorhersage')
parser.add_argument('--continue-train', action='store_true', help='Setze Training fort mit gegebenem Modell')
parser.add_argument('--epochs', type=int, default=None, help='Anzahl der Trainings-Epochen')

args = parser.parse_args()



if args.continue_train:
    continue_training(epochs=args.epochs)

if args.predict:
    write_prediction_to_json(IMAGE_PATH)




