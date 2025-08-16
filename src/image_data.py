import json
import os
from ultralytics import YOLO
import argparse

def write_data(data, model_name):
    data_path = f"Communication/{model_name}/data.json"
    os.makedirs(os.path.dirname(data_path), exist_ok=True)

    with open(data_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=4)

def create_new_model(model_name, data_set_type, yolo_model):
    model = YOLO(yolo_model)
    if data_set_type == "buildings":
        DATA_YAML = "dataset_buildings/data.yaml"
    else:
        DATA_YAML = "dataset_level/data.yaml"
    model.train(data=DATA_YAML, epochs=1, name=model_name)

def train_model(model_name, data_set_type, epochen):
    model_path = f"runs/detect/{model_name}/weights/best.pt"
    model = YOLO(model_path)
    if data_set_type == "buildings":
        DATA_YAML = "dataset_buildings/data.yaml"
    else:
        DATA_YAML = "dataset_level/data.yaml"

    model.train(
    data=DATA_YAML,
    epochs=epochen,
    imgsz=960,
    batch=16,                  # 16 passt meist auf 8GB VRAM (zur Not 8)
    workers=4,                 # CPU besser nutzen
    optimizer="SGD",           # stabiler als AdamW bei YOLO
    momentum=0.937,            # Standard-Optimalwert
    lr0=0.01,                  # höherer Start-LR für SGD
    lrf=0.01,                  # OneCycleLR fährt runter
    weight_decay=0.0005,
    patience=round(epochen*0.3),
    warmup_epochs=3,           # 2–5 ideal
    warmup_momentum=0.8,
    warmup_bias_lr=0.1,
    pretrained=True,
    amp=True,                  # Mixed Precision
    device=0,

    # Augmentation (deine Spezialsettings, nicht geändert):
    hsv_h=0.0,
    hsv_s=0.0,
    hsv_v=0.0,
    degrees=0.0,
    translate=0.05,
    scale=0.9,
    shear=0.0,
    perspective=0.0,
    flipud=0.0,
    fliplr=0.2,
    mosaic=0.5,
    mixup=0.0,
    copy_paste=0.0,

    save_period=10,
    exist_ok=True,
    val=True,
    project="runs/detect",
    name=model_name,
    cache="ram",
    ema=True,                  # stabilere final Weights
    cos_lr=True                # Cosine / OneCycle Scheduler
    )


    model.val(
    data=DATA_YAML,
    conf=0.3,
    imgsz=960,
    iou= 0.5,
    visualize= True,
    save=True,
    save_txt=True,
    save_conf=True,
    project="testvals",
    name=f"val_run_{model_name}",
    batch=16,
    plots=True,
    verbose=True,
    )

def testvals(model_name,data_set_type):
    model_path = f"runs/detect/{model_name}/weights/best.pt"
    if data_set_type == "buildings":
        DATA_YAML = "dataset_buildings/data.yaml"
    else:
        DATA_YAML = "dataset_level/data.yaml"

    model = YOLO(model_path)
    model.val(
    data=DATA_YAML,
    conf=0.3,
    imgsz=960,
    iou= 0.5,
    visualize= True,
    save=True,
    save_txt=True,
    save_conf=True,
    project="testvals",
    name=f"val_run_{model_name}",
    batch=16,
    plots=True,
    verbose=True,
    )



def write_prediction_to_json(model_name):
    image_path = f"Communication/{model_name}/screenshot.png"
    model_path = f"runs/detect/{model_name}/weights/best.pt"
    model = YOLO(model_path)
    results = model.predict(source=image_path, max_det= 999999999, conf=0.0)[0]
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

    write_data(output, model_name)

parser = argparse.ArgumentParser(description="Trainings- und Vorhersagemodus für YOLO Modell")

parser.add_argument('--zahl_erkennen', action='store_true', help='zahl erkennen')
parser.add_argument('--path', type=str, default=None, help='path zum image')
parser.add_argument('--create-model', action='store_true', help='Erstelle ein neues Modell mit einem bestimmten Namen')
parser.add_argument('--testvals', action='store_true', help='testvals')
parser.add_argument('--train', action='store_true', help='Starte ein neues Training')
parser.add_argument('--predict', action='store_true', help='Mache eine Vorhersage mit dem Modell')
parser.add_argument('--model-name', type=str, default=None, help='Name des Modells / Verzeichnisses')
parser.add_argument('--epochs', type=int, default=None, help='Anzahl der Trainings-Epochen')
parser.add_argument('--base', type=str, default=None, help='YOLO-Modellbasis (z. B. yolov8n.pt, yolov8s.pt)')
parser.add_argument('--dataset_type', type=str, default=None, help='')


args = parser.parse_args()

epochs = args.epochs

if args.testvals:
    testvals(args.model_name, args.dataset_type)


if args.create_model:
    create_new_model(args.model_name, args.dataset_type, args.base)


if args.train:
    train_model(args.model_name, args.dataset_type, epochs)


if args.predict:
    write_prediction_to_json(args.model_name)

