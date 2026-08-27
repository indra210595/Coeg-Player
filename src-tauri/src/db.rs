use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Folder {
    pub id: i64,
    pub path: String,
    pub created_at: String,
    pub file_count: i64,
    pub total_bytes: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub id: i64,
    pub folder_id: i64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub duration: i64,
    pub file_path: String,
    pub file_size: i64,
    pub cover_path: Option<String>,
    pub format: String,
    pub bitrate: i64,
    pub sample_rate: i64,
    pub bit_depth: i64,
    pub is_lossless: bool,
    pub waveform: Option<String>,
    pub lyrics: Option<String>,
    pub is_favorite: bool,
}

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}

pub fn init_db(app_handle: &tauri::AppHandle) -> Connection {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Gagal mengambil direktori AppData");

    let _ = std::fs::create_dir_all(&app_dir);
    let db_path = app_dir.join("music.db");
    let conn = Connection::open(db_path).expect("Gagal membuka database SQLite");

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;",

    )
    .expect("Gagal konfigurasi PRAGMA SQLite");

    let needs_reset = conn.prepare("SELECT is_favorite FROM songs LIMIT 1").is_err();
    if needs_reset {
        let _ = conn.execute("DROP TABLE IF EXISTS songs", []);
        let _ = conn.execute("DROP TABLE IF EXISTS folders", []);
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .unwrap();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS songs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            artist TEXT DEFAULT 'Unknown',
            album TEXT DEFAULT 'Unknown',
            genre TEXT DEFAULT 'Unknown',
            duration INTEGER DEFAULT 0,
            file_path TEXT UNIQUE NOT NULL,
            file_size INTEGER DEFAULT 0,
            cover_path TEXT,
            format TEXT DEFAULT 'MP3',
            bitrate INTEGER DEFAULT 0,
            sample_rate INTEGER DEFAULT 0,
            bit_depth INTEGER DEFAULT 16,
            is_lossless BOOLEAN DEFAULT 0,
            waveform TEXT,
            lyrics TEXT,
            is_favorite BOOLEAN DEFAULT 0,
            FOREIGN KEY (folder_id) REFERENCES folders (id) ON DELETE CASCADE
        )",
        [],
    )
    .unwrap();
    conn
}