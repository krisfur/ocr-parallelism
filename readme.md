# PaddleOCR Parallelisation with Python vs Rust

Testing if the GIL in Python has a tangible impact on parallelising PaddleOCR for perfoming OCR at scale.

## Results
Python only version: `Processed 1000 images in 42.62 seconds (23.46 images/sec)`

Rust parallelised version: `Processed 1000 images in 41.09 seconds (24.34 images/sec)`

This suggests it's not the GIL that's bottlenecking the process. Since PaddleOCR relies on C++ based PaddlePaddle for its backend the act of OCR itself sees no difference in performance and the GIL is used and released properly. 




While spawning processes in Python to make it parallelise cleanly is rather involved, the performance is realistically the same as with Rust doing it and having to call Python with `pyo3`. 




If one were to write a Rust application calling the compiled version of PaddleOCR itself with no Python hook there might be some minor improvement, but not at the scale warranting such development.

## Usage
You will need an `/images` folder with things to OCR in either case.

### Rust
Simply run `cargo run --release`.

### Python
Simply run `driver.py`