//Processed 100 images in 16.65 seconds (6.01 images/sec)
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rayon::prelude::*;
use std::cell::RefCell;
use std::fs;
use std::time::Instant;

// Thread-local PaddleOCR model instance
thread_local! {
    static OCR_INSTANCE: RefCell<Option<PyObject>> = RefCell::new(None);
}

// Initialize or get PaddleOCR instance per thread
fn get_or_init_ocr(py: Python<'_>) -> PyResult<PyObject> {
    OCR_INSTANCE.with(|cell| {
        let mut instance_opt = cell.borrow_mut();

        if let Some(ref instance) = *instance_opt {
            Ok(instance.clone_ref(py))
        } else {
            let ocr_module = py.import("paddleocr")?;
            let reader_class = ocr_module.getattr("PaddleOCR")?;

            // Build kwargs manually (no deprecated into_py calls)
            let kwargs = PyDict::new(py);
            kwargs.set_item("use_angle_cls", true)?;
            kwargs.set_item("lang", "en")?;

            let reader = reader_class.call((), Some(&kwargs))?;
            // Allow deprecation for this line until PyO3 finalizes the new API
            #[allow(deprecated)]
            let reader_obj = reader.into_py(py); // this is still fine in .24

            *instance_opt = Some(reader_obj.clone_ref(py));
            Ok(reader_obj)
        }
    })
}



// Run OCR on a single image
fn run_ocr(image_path: &str) -> PyResult<String> {
    Python::with_gil(|py| {
        let reader = get_or_init_ocr(py)?;
        let result = reader.bind(py).call_method1("ocr", (image_path,))?;
        Ok(format!("{:?}", result))
    })
}

fn main() -> PyResult<()> {
    // Limit Rayon to 4 threads for reproducibility
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .expect("Failed to build custom Rayon thread pool");

    // Gather image paths
    let image_paths: Vec<String> = fs::read_dir("images/")
        .unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()? == "jpg" {
                Some(path.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();

    let total = image_paths.len();
    println!("Processing {} images with 4 threads...", total);

    // Start timing
    let start = Instant::now();

    // Parallel processing
    image_paths.par_iter().for_each(|path| {
        match run_ocr(path) {
            Ok(_text) => {},//println!("{}: {}", path, text),
            Err(e) => eprintln!("Error on {}: {}", path, e),
        }
    });

    // Report duration
    let duration = start.elapsed();
    let seconds = duration.as_secs_f64();
    println!(
        "\nProcessed {} images in {:.2} seconds ({:.2} images/sec)",
        total,
        seconds,
        total as f64 / seconds
    );

    Ok(())
}
