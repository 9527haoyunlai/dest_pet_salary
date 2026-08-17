use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use tauri::State;

use crate::application::{
    AppSettingsDto, AppSnapshotDto, ApplicationContext, ApplicationError, ApplicationService,
    CollectionLedgerEntryDto, OfflineRewardBagDto,
};
use crate::persistence::SqliteRepository;

pub struct AppState {
    repository: Mutex<SqliteRepository>,
    context: ApplicationContext,
}

impl AppState {
    pub fn initialize(
        mut repository: SqliteRepository,
        context: ApplicationContext,
        at_utc: DateTime<Utc>,
    ) -> Result<Self, ApplicationError> {
        ApplicationService::new(&mut repository, &context).reconcile_offline(at_utc)?;
        Ok(Self {
            repository: Mutex::new(repository),
            context,
        })
    }

    fn repository(&self) -> Result<MutexGuard<'_, SqliteRepository>, String> {
        self.repository
            .lock()
            .map_err(|_| "application database lock is poisoned".to_owned())
    }
}

#[tauri::command]
pub fn get_app_snapshot(state: State<'_, AppState>) -> Result<AppSnapshotDto, String> {
    get_app_snapshot_at(&state, Utc::now())
}

pub fn get_app_snapshot_at(
    state: &AppState,
    at_utc: DateTime<Utc>,
) -> Result<AppSnapshotDto, String> {
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &state.context)
        .get_app_snapshot(at_utc)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_offline_reward_bags(
    state: State<'_, AppState>,
) -> Result<Vec<OfflineRewardBagDto>, String> {
    list_offline_reward_bags_from_state(&state)
}

pub fn list_offline_reward_bags_from_state(
    state: &AppState,
) -> Result<Vec<OfflineRewardBagDto>, String> {
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &state.context)
        .list_offline_reward_bags()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn claim_offline_reward_bag(
    state: State<'_, AppState>,
    bag_id: String,
) -> Result<CollectionLedgerEntryDto, String> {
    claim_offline_reward_bag_at(&state, &bag_id, Utc::now())
}

pub fn claim_offline_reward_bag_at(
    state: &AppState,
    bag_id: &str,
    at_utc: DateTime<Utc>,
) -> Result<CollectionLedgerEntryDto, String> {
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &state.context)
        .claim_offline_reward_bag(bag_id, at_utc)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_app_settings(state: State<'_, AppState>) -> Result<AppSettingsDto, String> {
    get_app_settings_from_state(&state)
}

pub fn get_app_settings_from_state(state: &AppState) -> Result<AppSettingsDto, String> {
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &state.context)
        .get_app_settings()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_app_settings(
    state: State<'_, AppState>,
    settings: AppSettingsDto,
) -> Result<AppSettingsDto, String> {
    update_app_settings_at(&state, settings, Utc::now())
}

pub fn update_app_settings_at(
    state: &AppState,
    settings: AppSettingsDto,
    at_utc: DateTime<Utc>,
) -> Result<AppSettingsDto, String> {
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &state.context)
        .update_app_settings(settings, at_utc)
        .map_err(|error| error.to_string())
}
