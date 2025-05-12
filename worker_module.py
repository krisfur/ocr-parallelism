import os
import logging
from paddleocr import PaddleOCR

# Preloaded model at module level
ocr = PaddleOCR(
    show_log=False,
    use_angle_cls=True,
    lang="en"
)
logging.getLogger("ppocr").setLevel(logging.INFO)

def ocr_image(image_path):
    try:
        result = ocr.ocr(image_path, cls=True)
        return (image_path, result)
    except Exception as e:
        logging.error(f"OCR failed on {image_path}: {e}")
        return (image_path, [])