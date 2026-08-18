use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, TimeDelta, Utc};
use tauri::State;

use crate::application::{
    AppSettingsDto, AppSnapshotDto, ApplicationContext, ApplicationEnvironment, ApplicationError,
    ApplicationService, CalendarMonthDto, CollectionLedgerEntryDto, LiveRewardEventDto,
    OfflineRewardBagDto, SalaryConfigurationDto, SalaryConfigurationService,
};
use crate::persistence::SqliteRepository;

pub struct AppState {
    repository: Mutex<SqliteRepository>,
    context: Mutex<Option<ApplicationContext>>,
    environment: ApplicationEnvironment,
    last_live_sync_at: Mutex<DateTime<Utc>>,
}

const LIVE_SESSION_STALE_AFTER_SECONDS: i64 = 15;

impl AppState {
    pub fn initialize(
        mut repository: SqliteRepository,
        context: ApplicationContext,
        at_utc: DateTime<Utc>,
    ) -> Result<Self, ApplicationError> {
        let mut service = ApplicationService::new(&mut repository, &context);
        service.package_pending_live_rewards(at_utc)?;
        service.reconcile_offline(at_utc)?;
        let environment =
            ApplicationEnvironment::from_calendar(context.cycle.timezone, context.calendar.clone());
        Ok(Self {
            repository: Mutex::new(repository),
            context: Mutex::new(Some(context)),
            environment,
            last_live_sync_at: Mutex::new(at_utc),
        })
    }

    pub fn open(
        mut repository: SqliteRepository,
        environment: ApplicationEnvironment,
        at_utc: DateTime<Utc>,
    ) -> Result<Self, ApplicationError> {
        let (_, context) = SalaryConfigurationService::new(&mut repository, &environment)
            .get_salary_configuration(at_utc)?;
        if let Some(context) = &context {
            let mut service = ApplicationService::new(&mut repository, context);
            service.package_pending_live_rewards(at_utc)?;
            service.reconcile_offline(at_utc)?;
        }
        Ok(Self {
            repository: Mutex::new(repository),
            context: Mutex::new(context),
            environment,
            last_live_sync_at: Mutex::new(at_utc),
        })
    }

    fn repository(&self) -> Result<MutexGuard<'_, SqliteRepository>, String> {
        self.repository
            .lock()
            .map_err(|_| "application database lock is poisoned".to_owned())
    }

    fn context(&self) -> Result<ApplicationContext, String> {
        self.context
            .lock()
            .map_err(|_| "application context lock is poisoned".to_owned())?
            .clone()
            .ok_or_else(|| ApplicationError::SalaryNotInitialized.to_string())
    }

    fn replace_context(&self, context: ApplicationContext) -> Result<(), String> {
        *self
            .context
            .lock()
            .map_err(|_| "application context lock is poisoned".to_owned())? = Some(context);
        Ok(())
    }
}

#[tauri::command]
pub fn sync_live_rewards(state: State<'_, AppState>) -> Result<Vec<LiveRewardEventDto>, String> {
    sync_live_rewards_at(&state, Utc::now())
}

pub fn sync_live_rewards_at(
    state: &AppState,
    at_utc: DateTime<Utc>,
) -> Result<Vec<LiveRewardEventDto>, String> {
    let context = state.context()?;
    let mut repository = state.repository()?;
    let mut last_sync = state
        .last_live_sync_at
        .lock()
        .map_err(|_| "live reward session clock lock is poisoned".to_owned())?;
    let inactive = at_utc.signed_duration_since(*last_sync)
        > TimeDelta::seconds(LIVE_SESSION_STALE_AFTER_SECONDS);
    let mut service = ApplicationService::new(&mut repository, &context);
    let events = if inactive {
        service
            .package_pending_live_rewards(at_utc)
            .map_err(|error| error.to_string())?;
        service
            .reconcile_offline(at_utc)
            .map_err(|error| error.to_string())?;
        service
            .list_pending_live_rewards()
            .map_err(|error| error.to_string())?
    } else {
        service
            .sync_live_rewards(at_utc)
            .map_err(|error| error.to_string())?
    };
    if at_utc > *last_sync {
        *last_sync = at_utc;
    }
    Ok(events)
}

#[tauri::command]
pub fn list_pending_live_rewards(
    state: State<'_, AppState>,
) -> Result<Vec<LiveRewardEventDto>, String> {
    list_pending_live_rewards_from_state(&state)
}

pub fn list_pending_live_rewards_from_state(
    state: &AppState,
) -> Result<Vec<LiveRewardEventDto>, String> {
    let context = state.context()?;
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &context)
        .list_pending_live_rewards()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn collect_live_reward(
    state: State<'_, AppState>,
    event_id: String,
) -> Result<CollectionLedgerEntryDto, String> {
    collect_live_reward_at(&state, &event_id, Utc::now())
}

pub fn collect_live_reward_at(
    state: &AppState,
    event_id: &str,
    at_utc: DateTime<Utc>,
) -> Result<CollectionLedgerEntryDto, String> {
    let context = state.context()?;
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &context)
        .collect_live_reward(event_id, at_utc)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_app_snapshot(state: State<'_, AppState>) -> Result<AppSnapshotDto, String> {
    get_app_snapshot_at(&state, Utc::now())
}

pub fn get_app_snapshot_at(
    state: &AppState,
    at_utc: DateTime<Utc>,
) -> Result<AppSnapshotDto, String> {
    let context = state.context()?;
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &context)
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
    let context = state.context()?;
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &context)
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
    let context = state.context()?;
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &context)
        .claim_offline_reward_bag(bag_id, at_utc)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_app_settings(state: State<'_, AppState>) -> Result<AppSettingsDto, String> {
    get_app_settings_from_state(&state)
}

pub fn get_app_settings_from_state(state: &AppState) -> Result<AppSettingsDto, String> {
    let context = state.context()?;
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &context)
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
    let context = state.context()?;
    let mut repository = state.repository()?;
    ApplicationService::new(&mut repository, &context)
        .update_app_settings(settings, at_utc)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_salary_configuration(
    state: State<'_, AppState>,
) -> Result<SalaryConfigurationDto, String> {
    get_salary_configuration_at(&state, Utc::now())
}

pub fn get_salary_configuration_at(
    state: &AppState,
    at_utc: DateTime<Utc>,
) -> Result<SalaryConfigurationDto, String> {
    let mut repository = state.repository()?;
    let (configuration, context) =
        SalaryConfigurationService::new(&mut repository, &state.environment)
            .get_salary_configuration(at_utc)
            .map_err(|error| error.to_string())?;
    drop(repository);
    if let Some(context) = context {
        state.replace_context(context)?;
    }
    Ok(configuration)
}

#[tauri::command]
pub fn initialize_salary(
    state: State<'_, AppState>,
    monthly_salary_exact: String,
) -> Result<SalaryConfigurationDto, String> {
    initialize_salary_at(&state, &monthly_salary_exact, Utc::now())
}

pub fn initialize_salary_at(
    state: &AppState,
    monthly_salary_exact: &str,
    at_utc: DateTime<Utc>,
) -> Result<SalaryConfigurationDto, String> {
    let mut repository = state.repository()?;
    let (configuration, context) =
        SalaryConfigurationService::new(&mut repository, &state.environment)
            .initialize_salary(monthly_salary_exact, at_utc)
            .map_err(|error| error.to_string())?;
    ApplicationService::new(&mut repository, &context)
        .reconcile_offline(at_utc)
        .map_err(|error| error.to_string())?;
    drop(repository);
    state.replace_context(context)?;
    Ok(configuration)
}

#[tauri::command]
pub fn update_next_cycle_salary(
    state: State<'_, AppState>,
    monthly_salary_exact: String,
) -> Result<SalaryConfigurationDto, String> {
    update_next_cycle_salary_at(&state, &monthly_salary_exact, Utc::now())
}

pub fn update_next_cycle_salary_at(
    state: &AppState,
    monthly_salary_exact: &str,
    at_utc: DateTime<Utc>,
) -> Result<SalaryConfigurationDto, String> {
    let mut repository = state.repository()?;
    SalaryConfigurationService::new(&mut repository, &state.environment)
        .update_next_cycle_salary(monthly_salary_exact, at_utc)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_calendar_month(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<CalendarMonthDto, String> {
    get_calendar_month_from_state(&state, year, month)
}

pub fn get_calendar_month_from_state(
    state: &AppState,
    year: i32,
    month: u32,
) -> Result<CalendarMonthDto, String> {
    let mut repository = state.repository()?;
    SalaryConfigurationService::new(&mut repository, &state.environment)
        .get_calendar_month(year, month)
        .map_err(|error| error.to_string())
}
