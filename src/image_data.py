from ultralytics import YOLO
import cv2

# YOLOv5s-Modell laden (automatischer Download)
model = YOLO("yolov5s.pt")

# Bild laden (z. B. mit OpenCV)
img = cv2.imread("dein_bild.jpg")

# Vorhersage
results = model(img)

# Ergebnisse anzeigen
results[0].show()  # Zeigt Bild mit Bounding Boxes

for result in results:
    for box in result.boxes:
        cls_id = int(box.cls[0])
        confidence = float(box.conf[0])
        x1, y1, x2, y2 = map(int, box.xyxy[0])
        print(f"Klasse: {model.names[cls_id]}, Konfidenz: {confidence:.2f}, Box: ({x1}, {y1}) – ({x2}, {y2})")


