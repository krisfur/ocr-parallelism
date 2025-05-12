//Processed all images in 41.09 seconds (24.34 images/sec)
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Instant;
use tempfile::NamedTempFile;

fn run_ocr(py: Python<'_>, image_path: &str) -> PyResult<()> {
    let reader = get_or_init_ocr(py)?;
    let _result = reader.bind(py).call_method1("ocr", (image_path,))?;
    Ok(())
}

thread_local! {
    static OCR_INSTANCE: std::cell::RefCell<Option<PyObject>> = std::cell::RefCell::new(None);
}

fn get_or_init_ocr(py: Python<'_>) -> PyResult<PyObject> {
    OCR_INSTANCE.with(|cell| {
        let mut instance_opt = cell.borrow_mut();

        if let Some(ref instance) = *instance_opt {
            Ok(instance.clone_ref(py))
        } else {
            let ocr_module = py.import("paddleocr")?;
            let reader_class = ocr_module.getattr("PaddleOCR")?;

            let kwargs = PyDict::new(py);
            kwargs.set_item("use_angle_cls", true)?;
            kwargs.set_item("lang", "en")?;

            let reader = reader_class.call((), Some(&kwargs))?;
            #[allow(deprecated)]
            let reader_obj = reader.into_py(py);

            *instance_opt = Some(reader_obj.clone_ref(py));
            Ok(reader_obj)
        }
    })
}

fn run_worker(input_file: &str) -> PyResult<()> {
    let file = File::open(input_file)?;
    let reader = BufReader::new(file);

    let image_paths: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    Python::with_gil(|py| {
        for path in image_paths {
            let _ = run_ocr(py, &path);
        }
    });

    Ok(())
}

fn run_controller() -> PyResult<()> {
    let image_dir = "images";
    let num_chunks = 6;

    let image_paths: Vec<_> = fs::read_dir(image_dir)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()? == "jpg" {
                Some(path.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();

    println!("Processing {} images using {} subprocesses...", image_paths.len(), num_chunks);
    let chunk_size = (image_paths.len() + num_chunks - 1) / num_chunks;

    let start = Instant::now();
    let exe_path = env::current_exe().expect("Cannot find current executable");
    let mut child_procs = vec![];

    for (_i, chunk) in image_paths.chunks(chunk_size).enumerate() {
        let mut tmp = NamedTempFile::new()?;
        for path in chunk {
            writeln!(tmp, "{}", path)?;
        }

        let tmp_path = tmp.path().to_path_buf();

        let child = Command::new(&exe_path)
            .arg("--worker")
            .arg("--input")
            .arg(tmp_path.to_string_lossy().to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to spawn worker process");

        // Keep temp file alive by storing it with the child
        child_procs.push((child, tmp));
    }

    for (mut child, _) in child_procs {
        child.wait().expect("Worker process failed");
    }

    let duration = start.elapsed();
    println!(
        "Processed all images in {:.2} seconds ({:.2} images/sec)",
        duration.as_secs_f64(),
        image_paths.len() as f64 / duration.as_secs_f64()
    );

    Ok(())
}


fn main() -> PyResult<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 3 && args[1] == "--worker" && args[2] == "--input" && args.len() >= 4 {
        return run_worker(&args[3]);
    } else {
        return run_controller();
    }
}
