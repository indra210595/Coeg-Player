mod commands;
mod db;
mod scanner;

use db::{init_db, AppState};
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            commands::toggle_favorite
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}