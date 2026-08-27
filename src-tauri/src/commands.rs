use crate::db::{AppState, Folder, Song};
use crate::scanner::scan_single_folder;
use crate::db::Playlist;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_folders(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.path, f.created_at, 
                    COUNT(s.id) as file_count, 
                    COALESCE(SUM(s.file_size), 0) as total_bytes 
             FROM folders f 
             LEFT JOIN songs s ON f.id = s.folder_id 
             GROUP BY f.id",
        )
        .map_err(|e| e.to_string())?;

    let folder_iter = stmt
        .query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                path: row.get(1)?,
                created_at: row.get(2)?,
                file_count: row.get(3)?,
                total_bytes: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut folders = Vec::new();
    for folder in folder_iter {
        folders.push(folder.map_err(|e| e.to_string())?);
    }
    Ok(folders)
}

#[tauri::command]
pub fn get_songs(state: State<'_, AppState>) -> Result<Vec<Song>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, folder_id, title, artist, album, genre, duration, file_path, file_size, cover_path, format, bitrate, sample_rate, bit_depth, is_lossless, waveform, lyrics, is_favorite FROM songs ORDER BY title ASC")
        .map_err(|e| e.to_string())?;

    let song_iter = stmt
        .query_map([], |row| {
            Ok(Song {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                album: row.get(4)?,
                genre: row.get(5)?,
                duration: row.get(6)?,
                file_path: row.get(7)?,
                file_size: row.get(8)?,
                cover_path: row.get(9)?,
                format: row.get(10)?,
                bitrate: row.get(11)?,
                sample_rate: row.get(12)?,
                bit_depth: row.get(13)?,
                is_lossless: row.get(14)?,
                waveform: row.get(15)?,
                lyrics: row.get(16)?,
                is_favorite: row.get(17)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut songs = Vec::new();
    for song in song_iter {
        songs.push(song.map_err(|e| e.to_string())?);
    }
    Ok(songs)
}

#[tauri::command]
pub fn add_folder(
    app_handle: tauri::AppHandle,
    folder_path: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let db_arc = state.db.clone();
    let folder_id = {
        let conn = db_arc.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO folders (path) VALUES (?1)",
            params![folder_path],
        )
        .map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT id FROM folders WHERE path = ?1")
            .map_err(|e| e.to_string())?;
        stmt.query_row(params![folder_path], |row| row.get(0))
            .map_err(|e| e.to_string())?
    };

    scan_single_folder(app_handle, folder_id, folder_path, db_arc);
    Ok(folder_id)
}

#[tauri::command]
pub fn sync_folder(
    app_handle: tauri::AppHandle,
    folder_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db_arc = state.db.clone();
    let folder_path: String = {
        let conn = db_arc.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT path FROM folders WHERE id = ?1",
            params![folder_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?
    };

    scan_single_folder(app_handle, folder_id, folder_path, db_arc);
    Ok(())
}

#[tauri::command]
pub fn delete_folder(folder_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    // ON DELETE CASCADE akan otomatis menghapus lagu terkait di tabel songs
    conn.execute("DELETE FROM folders WHERE id = ?1", params![folder_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn clear_database(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM folders", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(dead_code)]
#[tauri::command]
pub fn get_cover_bytes(path: String) -> Result<Vec<u8>, String> {
    let file_path = std::path::Path::new(&path);
    if !file_path.exists() {
        return Err(format!("File tidak ditemukan: {}", path));
    }
    std::fs::read(file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_favorite(song_id: i64, is_favorite: bool, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE songs SET is_favorite = ?1 WHERE id = ?2",
        params![is_favorite, song_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn create_playlist(name: String, state: State<'_, AppState>) -> Result<Playlist, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO playlists (name) VALUES (?1)",
        params![name.trim()],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    Ok(Playlist {
        id,
        name: name.trim().to_string(),
        created_at: "Baru saja".to_string(),
        song_count: 0,
    })
}

#[tauri::command]
pub fn get_playlists(state: State<'_, AppState>) -> Result<Vec<Playlist>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.created_at, COUNT(ps.song_id) as song_count 
             FROM playlists p 
             LEFT JOIN playlist_songs ps ON p.id = ps.playlist_id 
             GROUP BY p.id 
             ORDER BY p.id DESC",
        )
        .map_err(|e| e.to_string())?;

    let iter = stmt
        .query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                song_count: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for item in iter {
        list.push(item.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

#[tauri::command]
pub fn delete_playlist(playlist_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM playlists WHERE id = ?1", params![playlist_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn add_song_to_playlist(playlist_id: i64, song_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO playlist_songs (playlist_id, song_id) VALUES (?1, ?2)",
        params![playlist_id, song_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_song_from_playlist(playlist_id: i64, song_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
        params![playlist_id, song_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_playlist_songs(playlist_id: i64, state: State<'_, AppState>) -> Result<Vec<Song>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.folder_id, s.title, s.artist, s.album, s.genre, s.duration, s.file_path, s.file_size, s.cover_path, s.format, s.bitrate, s.sample_rate, s.bit_depth, s.is_lossless, s.waveform, s.lyrics, s.is_favorite 
             FROM songs s 
             INNER JOIN playlist_songs ps ON s.id = ps.song_id 
             WHERE ps.playlist_id = ?1 
             ORDER BY ps.rowid ASC",
        )
        .map_err(|e| e.to_string())?;

    let song_iter = stmt
        .query_map([playlist_id], |row| {
            Ok(Song {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                album: row.get(4)?,
                genre: row.get(5)?,
                duration: row.get(6)?,
                file_path: row.get(7)?,
                file_size: row.get(8)?,
                cover_path: row.get(9)?,
                format: row.get(10)?,
                bitrate: row.get(11)?,
                sample_rate: row.get(12)?,
                bit_depth: row.get(13)?,
                is_lossless: row.get(14)?,
                waveform: row.get(15)?,
                lyrics: row.get(16)?,
                is_favorite: row.get(17)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut songs = Vec::new();
    for song in song_iter {
        songs.push(song.map_err(|e| e.to_string())?);
    }
    Ok(songs)
}