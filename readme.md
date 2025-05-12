# PaddleOCR Parallelisation with Python vs Rust

Testing if the GIL in Python has a tangible impact on parallelising PaddleOCR for perfoming OCR at scale.

## Results
Python only version: `Processed 100 images in 10.71 seconds (9.34 images/sec)`

Rust parallelised version: `Processed 100 images in 16.65 seconds (6.01 images/sec)`

This suggests it's not the GIL that's bottlenecking the process. Since PaddleOCR relies on C++ based PaddlePaddle for its backend the act of OCR itself sees no difference in performance and the GIL is used and released properly. 




While spawning processes in Python to make it parallelise cleanly is rather involved, the performance is even better than with Rust doing it and having to call Python with `pyo3`.

## Usage
You will need an `/images` folder with things to OCR in either case.

### Rust
Simply run `cargo run --release`.

### Python
Simply run `driver.py`