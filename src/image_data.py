import json
from logging import raiseExceptions
import os
from ultralytics import YOLO

data=[]

def write_data(data):
    file_path = "Communication/data.json"
    os.makedirs(os.path.dirname(file_path), exist_ok=True)

    with open(file_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=4)

    print(f"Daten erfolgreich in {file_path} geschrieben.")



DATA_YAML = "dataset/data.yaml"
BEST_MODEL_PATH = 'runs/train/exp/weights/best.pt'


def train_new_model(epochs: int = 50):
    model = YOLO('yolov8n.pt')
    model.train(data=DATA_YAML, epochs=epochs)
    print("Training abgeschlossen. Das beste Modell findest du unter 'runs/train/exp/weights/best.pt'")


def continue_training(model_path=BEST_MODEL_PATH, epochs: int = 50):
    if not os.path.exists(model_path):
        print("Model path not found. In get_prediction()")
        return

    model = YOLO(BEST_MODEL_PATH)
    model.train(data=DATA_YAML, epochs=epochs)

    print("Weitertraining abgeschlossen.")


def get_prediction(image_path, model_path=BEST_MODEL_PATH):
    if not os.path.exists(model_path):
        print("Model path not found. In get_prediction()")
        return
    model = YOLO(model_path)
    results = model.predict(source=image_path)
    return results


write_data(data)





