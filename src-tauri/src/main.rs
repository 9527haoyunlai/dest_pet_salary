#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::Utc;
use salary_garden_core::application::ApplicationEnvironment;
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
            app.manage(AppState::open(
                repository,
                ApplicationEnvironment::default(),
                now,
            )?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            salary_garden_core::commands::get_app_snapshot,
            salary_garden_core::commands::list_offline_reward_bags,
            salary_garden_core::commands::claim_offline_reward_bag,
            salary_garden_core::commands::get_app_settings,
            salary_garden_core::commands::update_app_settings,
            salary_garden_core::commands::get_salary_configuration,
            salary_garden_core::commands::initialize_salary,
            salary_garden_core::commands::update_next_cycle_salary,
            salary_garden_core::commands::get_calendar_month,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Salary Garden");
}
