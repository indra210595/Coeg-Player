use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};
use walkdir::WalkDir;

#[derive(Clone, Serialize)]
pub struct ScanProgress {
    pub folder_id: i64,
    pub current: usize,
    pub total: usize,
    pub percentage: u8,
    pub status: String,
}

fn generate_waveform(seed_input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    seed_input.hash(&mut hasher);
    let mut seed = hasher.finish();

    let mut peaks = Vec::with_capacity(100);
    for i in 0..100 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let raw = (seed % 70) as u8 + 15;
        let envelope = match i {
            0..=10 => i as f32 / 10.0,
            85..=99 => (100 - i) as f32 / 15.0,
            _ => 1.0,
        };
        let val = ((raw as f32) * envelope).clamp(10.0, 100.0) as u8;
        peaks.push(val);
    }
    serde_json::to_string(&peaks).unwrap_or_else(|_| "[]".to_string())
}

fn extract_lyrics(file_path: &std::path::Path, tagged_file: Option<&lofty::file::TaggedFile>) -> Option<String> {
    let lrc_path = file_path.with_extension("lrc");
    if lrc_path.exists() {
        if let Ok(content) = std::fs::read_to_string(lrc_path) {
            if !content.trim().is_empty() {
                return Some(content);
            }
        }
    }

    if let Some(tf) = tagged_file {
        if let Some(tag) = tf.primary_tag().or_else(|| tf.first_tag()) {
            for item in tag.items() {
                let key_str = format!("{:?}", item.key()).to_lowercase();
                if key_str.contains("lyric") {
                    if let lofty::tag::ItemValue::Text(val) = item.value() {
                        if !val.trim().is_empty() {
                            return Some(val.clone());
                        }
                    }
                }
            }
        }
    }

    None
}

fn hash_image_bytes(bytes: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    let mut hasher = DefaultHasher::new();
    hasher.write(bytes);
    format!("{:x}", hasher.finish())
}

pub fn scan_single_folder(
    app: tauri::AppHandle,
    folder_id: i64,
    folder_path: String,
    db_arc: Arc<Mutex<Connection>>,
) {
    std::thread::spawn(move || {
        let covers_dir = app
            .path()
            .app_cache_dir()
            .map(|p| p.join("covers"))
            .ok();

        // 1. Ambil semua file yang tercatat di DB untuk folder ini
        let mut existing_songs = HashMap::new();

        if let Ok(conn) = db_arc.lock() {
            if let Ok(mut stmt) = conn.prepare("SELECT file_path, duration FROM songs WHERE folder_id = ?1") {
                if let Ok(path_iter) = stmt.query_map([folder_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))) {
                    for item in path_iter.flatten() {
                        existing_songs.insert(item.0, item.1);
                    }
                }
            }
        }

        let entries: Vec<_> = WalkDir::new(&folder_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();

        // 2. Kumpulkan path file aktual yang ada di disk saat ini
        let mut disk_paths = HashSet::new();
        for entry in &entries {
            let path = entry.path();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            if matches!(ext.as_str(), "mp3" | "flac" | "m4a" | "wav" | "ogg" | "opus") {
                disk_paths.insert(path.to_string_lossy().to_string());
            }
        }

        // 3. CLEANUP: Hapus dari DB lagu-lagu yang sudah DIHAPUS dari disk
        let deleted_paths: Vec<_> = existing_songs
            .keys()
            .filter(|p| !disk_paths.contains(*p))
            .cloned()
            .collect();

        if !deleted_paths.is_empty() {
            if let Ok(mut conn) = db_arc.lock() {
                if let Ok(tx) = conn.transaction() {
                    for del_path in &deleted_paths {
                        let _ = tx.execute("DELETE FROM songs WHERE file_path = ?1", params![del_path]);
                    }
                    let _ = tx.commit();
                }
            }
        }

        // 4. SCAN & INSERT/UPDATE LAGU DARI DISK
        let total_files = entries.len();
        let mut new_songs = Vec::new();

        for (index, entry) in entries.iter().enumerate() {
            let path = entry.path();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

            if matches!(ext.as_str(), "mp3" | "flac" | "m4a" | "wav" | "ogg" | "opus") {
                let file_path_str = path.to_string_lossy().to_string();
                let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0) as i64;

                let percentage = if total_files > 0 { ((index + 1) * 100 / total_files) as u8 } else { 100 };
                let _ = app.emit(
                    "scan-progress",
                    ScanProgress {
                        folder_id,
                        current: index + 1,
                        total: total_files,
                        percentage,
                        status: "syncing".to_string(),
                    },
                );

                // Jika file sudah ada di DB dan durasinya sudah terisi, lewati
                if let Some(&dur) = existing_songs.get(&file_path_str) {
                    if dur > 0 {
                        continue;
                    }
                }

                let mut title = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                let mut artist = "Unknown Artist".to_string();
                let mut album = "Unknown Album".to_string();
                let mut genre = "Unknown Genre".to_string();
                let mut duration = 0i64;
                let mut cover_path: Option<String> = None;
                let format_str = ext.to_uppercase();
                let is_lossless = matches!(ext.as_str(), "flac" | "wav" | "alac");
                let bitrate = 0i64;
                let sample_rate = 0i64;
                let bit_depth = 16i64;
                let waveform_data = generate_waveform(&file_path_str);
                let lyrics_data;

                let probed_file = Probe::open(path)
                    .ok()
                    .and_then(|p| p.guess_file_type().ok())
                    .and_then(|p| p.read().ok());

                if let Some(tagged_file) = probed_file {
                    lyrics_data = extract_lyrics(path, Some(&tagged_file));
                    duration = tagged_file.properties().duration().as_secs() as i64;

                    if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
                        if let Some(t) = tag.title() {
                            if !t.trim().is_empty() { title = t.to_string(); }
                        }
                        if let Some(a) = tag.artist() {
                            if !a.trim().is_empty() { artist = a.to_string(); }
                        }
                        if let Some(alb) = tag.album() {
                            if !alb.trim().is_empty() { album = alb.to_string(); }
                        }
                        if let Some(g) = tag.genre() {
                            if !g.trim().is_empty() { genre = g.to_string(); }
                        }

                        if let Some(covers_path) = &covers_dir {
                            let _ = std::fs::create_dir_all(covers_path);

                            if let Some(picture) = tag.pictures().first() {
                                let filename = format!("{}.jpg", hash_image_bytes(picture.data()));
                                let dest_path = covers_path.join(&filename);

                                if !dest_path.exists() {
                                    if let Ok(mut file) = File::create(&dest_path) {
                                        let _ = file.write_all(picture.data());
                                    }
                                }

                                if dest_path.exists() {
                                    cover_path = Some(dest_path.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                } else {
                    lyrics_data = extract_lyrics(path, None);
                }

                new_songs.push((
                    title, artist, album, genre, duration, file_path_str, file_size, cover_path, format_str, bitrate, sample_rate, bit_depth, is_lossless, waveform_data, lyrics_data,
                ));
            }
        }

        if !new_songs.is_empty() {
            if let Ok(mut conn) = db_arc.lock() {
                if let Ok(tx) = conn.transaction() {
                    for (title, artist, album, genre, duration, file_path, file_size, cover_path, format_str, bitrate, sample_rate, bit_depth, is_lossless, waveform_data, lyrics_data) in &new_songs {
                        let _ = tx.execute(
                            "INSERT INTO songs 
                            (folder_id, title, artist, album, genre, duration, file_path, file_size, cover_path, format, bitrate, sample_rate, bit_depth, is_lossless, waveform, lyrics) 
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                            ON CONFLICT(file_path) DO UPDATE SET 
                                duration = excluded.duration",
                            params![folder_id, title, artist, album, genre, duration, file_path, file_size, cover_path, format_str, bitrate, sample_rate, bit_depth, is_lossless, waveform_data, lyrics_data],
                        );
                    }
                    let _ = tx.commit();
                }
            }
        }

        let _ = app.emit(
            "scan-progress",
            ScanProgress {
                folder_id,
                current: total_files,
                total: total_files,
                percentage: 100,
                status: "synced".to_string(),
            },
        );

        let _ = app.emit("scan-finished", folder_id);
    });
}