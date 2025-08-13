import json
import os
from ultralytics import YOLO
import argparse
# import cv2
# import pytesseract

# def read_number(path):
#     img = cv2.imread(path)
#
#     gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
#     _, thresh = cv2.threshold(gray, 120, 255, cv2.THRESH_BINARY)
#
#     custom_config = r'--oem 3 --psm 6 outputbase digits'
#     text = pytesseract.image_to_string(thresh, config=custom_config)
#
#     return text




def write_data(data):
    print("Hallo hier ist python. Ich schreibe jetzt diese data in data.json: ", data)
    file_path = "Communication/data.json"
    os.makedirs(os.path.dirname(file_path), exist_ok=True)
    with open(file_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=4)
    print(f"JSON Data erfolgreich in {file_path} geschrieben.")

IMAGE_PATH = "Communication/screenshot.png"

def create_new_model(model_name, data_set_type, yolo_model):
    model = YOLO(yolo_model)
    if data_set_type == "buildings":
        DATA_YAML = "dataset_buildings/data.yaml"
    else:
        DATA_YAML = "dataset_level/data.yaml"
    model.train(data=DATA_YAML, epochs=1, name=model_name, augment=True)
    print(f"Erstellung von '{model_name}'abgeschlossen. Das Modell findest du unter 'runs/detect/{model_name}/weights/best.pt'")

def train_model(model_name, data_set_type, epochen):
    print("Starte Training.")
    model_path = f"runs/detect/{model_name}/weights/best.pt"

    model = YOLO(model_path)
    if data_set_type == "buildings":
        DATA_YAML = "dataset_buildings/data.yaml"
    else:
        DATA_YAML = "dataset_level/data.yaml"



    model.train(
    data=DATA_YAML,
    epochs=epochen,
    imgsz=960,                 # Reicht meistens, 1280 wäre overkill
    batch=1,                   # je nach VRAM
         #accumulate=8, #nur für die ubutuntu user
       # workers=1,

    optimizer="AdamW",
    lr0=0.001,
    lrf=0.01,
    weight_decay=0.0005,
    patience=round(epochen*0.3), #erstmal soll er undendlich lang trainieren
    warmup_epochs=50,#round(epochen*0.05),
    pretrained=True,
        amp=True,
      device=0, # geht nicht auf mac

    # Augmentation (angepasst!):
    hsv_h=0.0,                 # Keine Farbanpassung!
    hsv_s=0.0,
    hsv_v=0.0,
    degrees=0.0,               # Keine Rotation nötig
    translate=0.05,            # Leichte Verschiebung erlaubt
    scale=0.9,                 # Wenig Skalierung
    shear=0.0,                 # Keine Scherung
    perspective=0.0,           # Keine Verzerrung
    flipud=0.0,                # Kein vertikales Flip
    fliplr=0.2,                # Nur horizontales Flip (wenn sinnvoll)
    mosaic=0.5,                # Optional – hilft evtl. bei Generalisierung
    mixup=0.0,                 # Nicht sinnvoll bei UI-Bildern
    copy_paste=0.0,            # Ebenfalls ungeeignet

    save_period=10,
    exist_ok=True,
    val=True,
    project="runs/detect",
    name=model_name,
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

    print("Training erfolgreich abgeschlossen.")

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



def write_prediction_to_json(model_name, image_path):

    model_path = f"runs/detect/{model_name}/weights/best.pt"

    model = YOLO(model_path)
    # model = YOLO("yolov8n")

    results = model.predict(source=image_path, max_det= 999999999, conf=0.0)[0]

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
    print("Output in python nach dem parsen des results von dem model. Das hier sollte jetzt in data.json geschrieben werden: ", output)

    write_data(output)

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
    write_prediction_to_json(args.model_name, "Communication/screenshot.png")

