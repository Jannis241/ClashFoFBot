import json
import os

def write_data(data):
    file_path = "Communication/data.json"
    os.makedirs(os.path.dirname(file_path), exist_ok=True)

    with open(file_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=4)

    print(f"Daten erfolgreich in {file_path} geschrieben.")



data = []


from ultralytics import YOLO

model = YOLO('yolov8n.yaml')

model.train(
    data='dataset/data.yaml',
    epochs=30,
    imgsz=640,
    batch=8
)

model.val()

best_model = YOLO('runs/detect/train/weights/best.pt')
results = best_model('Communication/screenshot.png', show=True)

print(results)


write_data(data)





