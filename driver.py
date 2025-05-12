#Processed 1000 images in 42.62 seconds (23.46 images/sec)
import os
import time
import logging
import multiprocessing as mp
from worker_module import ocr_image

def main():
    IMAGE_DIR = "images"
    NUM_PROCESSES = 4

    try:
        mp.set_start_method("spawn")
    except RuntimeError as e:
        if "context has already been set" not in str(e):
            raise

    image_paths = [
        os.path.join(IMAGE_DIR, f)
        for f in os.listdir(IMAGE_DIR)
        if f.lower().endswith(".jpg")
    ]

    total = len(image_paths)
    print(f"Processing {total} images with {NUM_PROCESSES} processes...")

    start = time.time()

    with mp.Pool(NUM_PROCESSES) as pool:
        results = pool.map(ocr_image, image_paths)

    duration = time.time() - start
    #for path, result in results: #uncommment to see OCR results
    #    print(f"{path}: {result}")

    print(
        f"\nProcessed {total} images in {duration:.2f} seconds "
        f"({total / duration:.2f} images/sec)"
    )

if __name__ == "__main__":
    main()
