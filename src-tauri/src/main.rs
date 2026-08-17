#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use salary_garden_core::persistence::SqliteRepository;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let repository = SqliteRepository::open(app_data_dir.join("salary-garden.sqlite3"))?;
            app.manage(Mutex::new(repository));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Salary Garden");
}
