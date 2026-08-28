mod commands;
mod db;
mod scanner;

use db::{init_db, AppState};
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let conn = init_db(app.handle());
            app.manage(AppState {
                db: Arc::new(Mutex::new(conn)),
            });
            Ok(())
        })
       .invoke_handler(tauri::generate_handler![
            commands::get_folders,
            commands::get_songs,
            commands::add_folder,
            commands::sync_folder,
            commands::delete_folder,
            commands::clear_database,
            commands::get_cover_bytes,
            commands::toggle_favorite,
            commands::create_playlist,
            commands::get_playlists,
            commands::delete_playlist,
            commands::add_song_to_playlist,
            commands::remove_song_from_playlist,
            commands::get_playlist_songs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}