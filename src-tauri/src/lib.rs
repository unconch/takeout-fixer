mod engine;

use engine::{ScanResult, ProgressEvent, RepairReport, is_video_file};
use tauri::{Emitter, Runtime};

#[tauri::command]
fn analyze_folder(path: String) -> Vec<ScanResult> {
    engine::analyze(&path)
}

#[tauri::command]
async fn repair_all<R: Runtime>(
    app: tauri::AppHandle<R>,
    files: Vec<ScanResult>,
    dest_path: String,
    copy_solos: bool,
) -> Result<RepairReport, String> {
    let total = files.len();
    let mut report = RepairReport {
        fixed_photos: 0,
        fixed_videos: 0,
        gps_restored: 0,
        solo_copied: 0,
        failed: 0,
    };

    for (index, file) in files.into_iter().enumerate() {
        let is_video = is_video_file(std::path::Path::new(&file.media_file));
        let filename = std::path::Path::new(&file.media_file)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Emit progress before starting each file
        let _ = app.emit("repair-progress", ProgressEvent {
            current: index + 1,
            total,
            filename: filename.clone(),
            status: "Processing...".to_string(),
        });

        match engine::repair_single(&file.media_file, file.metadata_file.as_ref(), &dest_path, copy_solos) {
            Ok(gps_restored) => {
                if file.metadata_file.is_some() {
                    if is_video {
                        report.fixed_videos += 1;
                    } else {
                        report.fixed_photos += 1;
                    }
                    if gps_restored {
                        report.gps_restored += 1;
                    }
                } else if copy_solos {
                    report.solo_copied += 1;
                }
            }
            Err(_) => {
                report.failed += 1;
            }
        }
    }

    Ok(report)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![analyze_folder, repair_all])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
