import { invoke } from "@tauri-apps/api/core";

import type {
  AppSettingsDto,
  AppSnapshotDto,
  CalendarMonthDto,
  CollectionLedgerEntryDto,
  LiveRewardEventDto,
  OfflineRewardBagDto,
  SalaryConfigurationDto,
} from "./types";

export function getAppSnapshot(): Promise<AppSnapshotDto> {
  return invoke<AppSnapshotDto>("get_app_snapshot");
}

export function listOfflineRewardBags(): Promise<OfflineRewardBagDto[]> {
  return invoke<OfflineRewardBagDto[]>("list_offline_reward_bags");
}

export function claimOfflineRewardBag(
  bagId: string,
): Promise<CollectionLedgerEntryDto> {
  return invoke<CollectionLedgerEntryDto>("claim_offline_reward_bag", {
    bagId,
  });
}

export function getAppSettings(): Promise<AppSettingsDto> {
  return invoke<AppSettingsDto>("get_app_settings");
}

export function updateAppSettings(
  settings: AppSettingsDto,
): Promise<AppSettingsDto> {
  return invoke<AppSettingsDto>("update_app_settings", { settings });
}

export function getSalaryConfiguration(): Promise<SalaryConfigurationDto> {
  return invoke<SalaryConfigurationDto>("get_salary_configuration");
}

export function initializeSalary(
  monthlySalaryExact: string,
): Promise<SalaryConfigurationDto> {
  return invoke<SalaryConfigurationDto>("initialize_salary", {
    monthlySalaryExact,
  });
}

export function updateNextCycleSalary(
  monthlySalaryExact: string,
): Promise<SalaryConfigurationDto> {
  return invoke<SalaryConfigurationDto>("update_next_cycle_salary", {
    monthlySalaryExact,
  });
}

export function getCalendarMonth(
  year: number,
  month: number,
): Promise<CalendarMonthDto> {
  return invoke<CalendarMonthDto>("get_calendar_month", { year, month });
}

export function syncLiveRewards(): Promise<LiveRewardEventDto[]> {
  return invoke<LiveRewardEventDto[]>("sync_live_rewards");
}

export function listPendingLiveRewards(): Promise<LiveRewardEventDto[]> {
  return invoke<LiveRewardEventDto[]>("list_pending_live_rewards");
}

export function collectLiveReward(
  eventId: string,
): Promise<CollectionLedgerEntryDto> {
  return invoke<CollectionLedgerEntryDto>("collect_live_reward", { eventId });
}
