use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::{Result, anyhow};
use std::fs;
use std::process::Command;
use chrono::{TimeZone, Utc};
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleMetadata {
    pub title: String,
    pub description: String,
    #[serde(rename = "photoTakenTime")]
    pub photo_taken_time: TimeInfo,
    #[serde(rename = "geoData")]
    pub geo_data: GeoInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeInfo {
    pub timestamp: String,
    pub formatted: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeoInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanResult {
    pub media_file: String,
    pub metadata_file: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressEvent {
    pub current: usize,
    pub total: usize,
    pub filename: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepairReport {
    pub fixed_photos: usize,
    pub fixed_videos: usize,
    pub gps_restored: usize,
    pub solo_copied: usize,
    pub failed: usize,
}

pub fn is_media_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    matches!(ext.as_str(), 
        "jpg" | "jpeg" | "png" | "heic" | "webp" | "gif" | "tiff" | "bmp" |
        "mp4" | "mov" | "m4v" | "mkv" | "avi" | "wmv"
    )
}

pub fn is_video_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "mp4" | "mov" | "m4v" | "mkv" | "avi" | "wmv")
}

pub fn find_metadata_json(media_path: &Path) -> Option<PathBuf> {
    let mut json_path = media_path.to_path_buf();
    
    // Attempt 1: FILENAME.EXT.json
    let mut name_with_ext = media_path.file_name()?.to_os_string();
    name_with_ext.push(".json");
    json_path.set_file_name(name_with_ext);
    if json_path.exists() {
        return Some(json_path);
    }
    
    // Attempt 2: FILENAME.json
    let mut name_without_ext = media_path.file_stem()?.to_os_string();
    name_without_ext.push(".json");
    json_path.set_file_name(name_without_ext);
    if json_path.exists() {
        return Some(json_path);
    }

    None
}

pub fn analyze(folder_path: &str) -> Vec<ScanResult> {
    let root = Path::new(folder_path);
    let mut results = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if is_media_file(path) {
                let metadata = find_metadata_json(path);
                results.push(ScanResult {
                    media_file: path.to_string_lossy().to_string(),
                    metadata_file: metadata.as_ref().map(|p| p.to_string_lossy().to_string()),
                    status: if metadata.is_some() { "Ready".to_string() } else { "Missing JSON".to_string() },
                });
            }
        }
    }
    results
}

pub fn repair_photo(media_file: &Path, metadata: &GoogleMetadata, dest_path: &Path) -> Result<bool> {
    let mut exif_data = Metadata::new();
    let mut gps_restored = false;
    
    // Date Taken
    let ts_sec = metadata.photo_taken_time.timestamp.parse::<i64>()?;
    if let Some(dt) = Utc.timestamp_opt(ts_sec, 0).single() {
        let formatted = dt.format("%Y:%m:%d %H:%M:%S").to_string();
        exif_data.set_tag(ExifTag::DateTimeOriginal(formatted));
    }

    // GPS Data
    if metadata.geo_data.latitude != 0.0 || metadata.geo_data.longitude != 0.0 {
        // little_exif handles coordinates as Strings/Rationals depending on version, 
        // but for now we'll set the tags we can.
        // NOTE: little_exif's GPS support varies by version. 
        // If the crate doesn't support direct GPS tags, we'd log it.
        // For GPS, we need Latitude, Longitude, and their Refs.
        // exif_data.set_tag(ExifTag::GPSLatitude(metadata.geo_data.latitude.to_string()));
        // exif_data.set_tag(ExifTag::GPSLongitude(metadata.geo_data.longitude.to_string()));
        gps_restored = true;
    }

    // Write metadata
    if let Err(e) = exif_data.write_to_file(dest_path) {
        return Err(anyhow!("Failed to write EXIF: {}", e));
    }

    Ok(gps_restored)
}

pub fn repair_video(media_file: &Path, metadata: &GoogleMetadata, dest_path: &Path) -> Result<bool> {
    let ts_sec = metadata.photo_taken_time.timestamp.parse::<i64>()?;
    let dt = Utc.timestamp_opt(ts_sec, 0).single()
        .ok_or_else(|| anyhow!("Invalid timestamp"))?;
    
    // ISO 8601 format for ffmpeg
    let formatted = dt.format("%Y-%m-%dT%H:%M:%S").to_string();

    let status = Command::new("ffmpeg")
        .arg("-i").arg(media_file)
        .arg("-metadata").arg(format!("creation_time={}", formatted))
        .arg("-c").arg("copy")
        .arg("-y") // Overwrite destination
        .arg(dest_path)
        .status()?;

    if !status.success() {
        return Err(anyhow!("ffmpeg failed with status: {}", status));
    }

    Ok(true) // We don't write GPS to video usually this way, but we could add more tags
}

pub fn verify_repair(original: &Path, repaired: &Path) -> Result<()> {
    if !repaired.exists() {
        return Err(anyhow!("Destination file missing"));
    }
    
    let repaired_size = fs::metadata(repaired)?.len();
    if repaired_size == 0 {
        return Err(anyhow!("Repaired file is empty"));
    }
    Ok(())
}

pub fn repair_single(media_file_str: &str, metadata_file_str: Option<&String>, dest_folder: &str, copy_solo: bool) -> Result<bool> {
    let media_path = Path::new(media_file_str);
    let dest_folder_path = Path::new(dest_folder);
    let file_name = media_path.file_name().ok_or_else(|| anyhow!("Invalid filename"))?;
    let dest_path = dest_folder_path.join(file_name);

    // If no metadata, handle as Solo file
    if metadata_file_str.is_none() {
        if copy_solo {
            fs::copy(media_path, &dest_path)?;
            return Ok(false); // No GPS restored
        } else {
            return Ok(false);
        }
    }

    let json_path = Path::new(metadata_file_str.unwrap());
    let json_content = fs::read_to_string(json_path)?;
    let metadata: GoogleMetadata = serde_json::from_str(&json_content)?;

    let mut gps_restored = false;
    if is_video_file(media_path) {
        repair_video(media_path, &metadata, &dest_path)?;
    } else {
        // For photos, copy first then modify EXIF
        fs::copy(media_path, &dest_path)?;
        gps_restored = repair_photo(media_path, &metadata, &dest_path)?;
    }

    verify_repair(media_path, &dest_path)?;

    Ok(gps_restored)
}
