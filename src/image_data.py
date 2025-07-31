import json
import os
from ultralytics import YOLO
import argparse
import cv2
import pytesseract



def read_number(path):
    img = cv2.imread(path)

    gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
    _, thresh = cv2.threshold(gray, 120, 255, cv2.THRESH_BINARY)

    custom_config = r'--oem 3 --psm 6 outputbase digits'
    text = pytesseract.image_to_string(thresh, config=custom_config)

    return text




def write_data(data):
    file_path = "Communication/data.json"
    os.makedirs(os.path.dirname(file_path), exist_ok=True)
    with open(file_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=4)
    print(f"JSON Data erfolgreich in {file_path} geschrieben.")

IMAGE_PATH = "Communication/screenshot.png"
DATA_YAML = "dataset/data.yaml"

def create_new_model(model_name, yolo_model):
    model = YOLO(yolo_model)
    model.train(data=DATA_YAML, epochs=1, name=model_name, augment=True)
    print(f"Erstellung von '{model_name}'abgeschlossen. Das Modell findest du unter 'runs/detect/{model_name}/weights/best.pt'")

def train_model(model_name, epochen):
    model_path = f"runs/detect/{model_name}/weights/best.pt"

    model = YOLO(model_path)
    model.train(data=DATA_YAML, epochs=epochen, name=model_name, exist_ok=True,augment=True)

    print("Training erfolgreich abgeschlossen.")



def write_prediction_to_json(model_name, image_path):

    model_path = f"runs/detect/{model_name}/weights/best.pt"

    model = YOLO(model_path)

    results = model.predict(source=image_path)[0]

   # results.show()

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

    write_data(output)

parser = argparse.ArgumentParser(description="Trainings- und Vorhersagemodus für YOLO Modell")

parser.add_argument('--zahl_erkennen', action='store_true', help='zahl erkennen')
parser.add_argument('--path', type=str, default=None, help='path zum image')
parser.add_argument('--create-model', action='store_true', help='Erstelle ein neues Modell mit einem bestimmten Namen')
parser.add_argument('--train', action='store_true', help='Starte ein neues Training')
parser.add_argument('--predict', action='store_true', help='Mache eine Vorhersage mit dem Modell')
parser.add_argument('--model-name', type=str, default=None, help='Name des Modells / Verzeichnisses')
parser.add_argument('--epochs', type=int, default=None, help='Anzahl der Trainings-Epochen')
parser.add_argument('--base', type=str, default=None, help='YOLO-Modellbasis (z. B. yolov8n.pt, yolov8s.pt)')


args = parser.parse_args()

epochs = args.epochs

if args.zahl_erkennen:
    text = read_number(args.path)
    with open("Communication/number.txt", 'w', encoding='utf-8') as f:
        f.write(text)

if args.create_model:
    create_new_model(args.model_name, args.base)


if args.train:
    train_model(args.model_name, epochs)

if args.predict:
    write_prediction_to_json(args.model_name, "Communication/screenshot.png")

