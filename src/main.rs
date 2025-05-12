//Processed 100 images in 11.12 seconds (9.00 images/sec)
//Processed 1000 images in 65.68 seconds (15.23 images/sec)
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
fn run_ocr(py: Python<'_>, image_path: &str) -> PyResult<String> {
    let sys = py.import("sys")?;
    let io = py.import("io")?;

    // Backup original stdout/stderr
    let original_stdout = sys.getattr("stdout")?;
    let original_stderr = sys.getattr("stderr")?;

    // Redirect to dummy StringIO
    let fake_output = io.call_method0("StringIO")?;
    sys.setattr("stdout", &fake_output)?;
    sys.setattr("stderr", &fake_output)?;

    // Run OCR
    let result = (|| -> PyResult<String> {
        let reader = get_or_init_ocr(py)?;
        let ocr_result = reader
            .bind(py)
            .call_method1("ocr", (image_path,))?;
        Ok(format!("{:?}", ocr_result))
    })();

    // Restore original stdout/stderr
    sys.setattr("stdout", original_stdout)?;
    sys.setattr("stderr", original_stderr)?;

    result
}

fn main() -> PyResult<()> {
    unsafe {
        std::env::set_var("OMP_NUM_THREADS", "2");
        std::env::set_var("CPU_NUM_THREADS", "2");
    }
    // Limit Rayon to 4 threads for reproducibility
    rayon::ThreadPoolBuilder::new()
        .num_threads(6)
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
        Python::with_gil(|py| {
            match run_ocr(py, path) {
                Ok(_) => {}, //for zero output
                //Ok(text) => println!("{}: {}", path, text), //for printing results
                Err(e) => eprintln!("Error on {}: {}", path, e),
            }
        });
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
