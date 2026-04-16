use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::thread;
use tauri::{AppHandle, Emitter};

use crate::app_identity;
use crate::ocr;

const VALID_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "tiff", "tif"];

/// Start watching for new image files in Scans directory
pub fn start_watcher(app_handle: AppHandle) -> Result<(), String> {
    let candidates: Vec<PathBuf> = vec![
        app_identity::scans_dir(),
        std::env::current_dir().unwrap_or_default().join("Scans"),
        std::env::current_dir()
            .unwrap_or_default()
            .join("..")
            .join("Scans"),
    ];

    let scans_dir = candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone());

    std::fs::create_dir_all(&scans_dir)
        .map_err(|e| format!("Failed to create Scans directory: {}", e))?;

    let scans_path = scans_dir.clone();

    thread::spawn(move || {
        if let Err(e) = run_watcher(scans_path, app_handle) {
            eprintln!("File watcher error: {}", e);
        }
    });

    println!("File watcher started on: {}", scans_dir.display());
    Ok(())
}

fn run_watcher(scans_dir: PathBuf, app_handle: AppHandle) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    watcher
        .watch(&scans_dir, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to watch directory: {}", e))?;

    // Initialize OCR engine once
    let engine = match ocr::create_engine() {
        Ok(e) => e,
        Err(e) => {
            eprintln!(
                "OCR engine not available: {}. Watcher running without OCR.",
                e
            );
            loop {
                let _ = rx.recv();
            }
        }
    };

    println!("OCR engine ready. Waiting for scans...");

    for event in rx {
        if matches!(event.kind, EventKind::Create(_)) {
            for path in event.paths {
                if is_valid_image(&path) {
                    thread::sleep(std::time::Duration::from_millis(500));
                    println!("New scan detected: {}", path.display());

                    match ocr::ocr_image(&engine, &path) {
                        Ok(lines) => {
                            let cccd = ocr::parse_cccd(&lines);
                            println!("OCR result: {:?}", cccd);
                            let _ = app_handle.emit("ocr-result", &cccd);
                        }
                        Err(e) => {
                            eprintln!("OCR failed for {}: {}", path.display(), e);
                            let _ = app_handle.emit("ocr-error", &e);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn is_valid_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| VALID_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}
