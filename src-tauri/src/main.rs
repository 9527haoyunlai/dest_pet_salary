#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::Utc;
use salary_garden_core::application::ApplicationContext;
use salary_garden_core::commands::AppState;
use salary_garden_core::persistence::SqliteRepository;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let repository = SqliteRepository::open(app_data_dir.join("salary-garden.sqlite3"))?;
            let now = Utc::now();
            let context = ApplicationContext::default_for(now)?;
            app.manage(AppState::initialize(repository, context, now)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            salary_garden_core::commands::get_app_snapshot,
            salary_garden_core::commands::list_offline_reward_bags,
            salary_garden_core::commands::claim_offline_reward_bag,
            salary_garden_core::commands::get_app_settings,
            salary_garden_core::commands::update_app_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Salary Garden");
}
