import requests
import base64
from io import BytesIO
from PIL import Image

# Адрес твоего бэкенда
url = "https://rbackend.hackhaton.vladyalk.ru/api/internal/hash"

# GET или POST — в зависимости от того, как сервер отдаёт картинки
response = requests.post(url)  # или requests.post(url, json={...})

if response.status_code != 200:
    raise RuntimeError(f"Ошибка запроса: {response.status_code}")

# Парсим JSON
data = response.json()


# Функция для показа base64-картинки
def show_base64_image(b64_str, title="Image"):
    img_data = base64.b64decode(b64_str)
    img = Image.open(BytesIO(img_data))
    img.show(title=title)


# Показываем все три картинки
show_base64_image(data["frame_basic"], "Frame Base")
show_base64_image(data["frame_grayscale"], "Frame Grayscale")
show_base64_image(data["frame_downscale"], "Frame Downscale")
