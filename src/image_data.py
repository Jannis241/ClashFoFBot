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

def create_new_model(model_name, yolo_model):
    print(f"Creating new model '{model_name}' with model {yolo_model}")
    model = YOLO(yolo_model)
    model.train(data=DATA_YAML, epochs=1, name=model_name)
    print(f"Erstellung von '{model_name}'abgeschlossen. Das Modell findest du unter 'runs/detect/{model_name}/weights/best.pt'")

def train_model(model_name, epochen):
    print(f"Training model '{model_name}'")
    model_path = f"runs/detect/{model_name}/weights/best.pt"
    if not os.path.exists(model_path):
        print(f"Modell {model_name} wurde nicht unter {model_path} gefunden. Training wird abgebrochen..")
        return

    print("Modell gefunden:", model_path)
    print(f"Starte training.. ({epochen}Epochen)")
    model = YOLO(model_path)
    model.train(data=DATA_YAML, epochs=epochen, name = "fufu")
    print("Training erfolgreich abgeschlossen.")

def delete_model(model_name):
    print(f"Trying to delete model '{model_name}'")
    model_path = f"runs/detect/{model_name}/weights/best.pt"

    if os.path.exists(model_path):
        print(f"Deleting model '{model_name}' from {model_path}")
        os.remove(model_path)
        return

    print(f"Failed: Modell '{model_name}' nicht gefunden. Stelle sicher, dass das model unter {model_path} exisitert.")


def write_prediction_to_json(image_path, model_name):

    model_path = f"runs/detect/{model_name}/weights/best.pt"

    if not os.path.exists(model_path):
        print(f"Modell {model_name} wurde nicht unter {model_path} gefunden. Stelle sicher, dass das Model '{model_name}' unter {model_path} liegt.")
        return

    print(f"Modell für die prediction: {model_name} ({model_path})")
    print("Predicte: ", image_path)

    model = YOLO(model_path)

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
    print("Output des Modells: ", output)

    write_data(output)

parser = argparse.ArgumentParser(description="Trainings- und Vorhersagemodus für YOLO Modell")

parser.add_argument('--create-model', action='store_true', help='Erstelle ein neues Modell mit einem bestimmten Namen')
parser.add_argument('--train', action='store_true', help='Starte ein neues Training')
parser.add_argument('--predict', action='store_true', help='Mache eine Vorhersage mit dem Modell')
parser.add_argument('--delete-model', action='store_true', help='Lösche ein Modellverzeichnis')
parser.add_argument('--model-name', type=str, default=None, help='Name des Modells / Verzeichnisses')
parser.add_argument('--epochs', type=int, default=None, help='Anzahl der Trainings-Epochen')
parser.add_argument('--base', type=str, default=None, help='YOLO-Modellbasis (z. B. yolov8n.pt, yolov8s.pt)')
parser.add_argument('--image_path', type=str, default=None, help='Image path')


args = parser.parse_args()

epochs = args.epochs

if args.create_model:
    create_new_model(args.model_name, args.base)


if args.train:
    train_model(args.model_name, epochs)

if args.predict:
    write_prediction_to_json(args.model_name, args.image_path)

if args.delete_model:
    delete_model(args.model_name)
