import { invoke } from "@tauri-apps/api/core";

import type {
  AppSettingsDto,
  AppSnapshotDto,
  CollectionLedgerEntryDto,
  OfflineRewardBagDto,
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
