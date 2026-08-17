export type ExactDecimal = string;

export interface RewardCountsDto {
  silver: number;
  gold: number;
  diamond: number;
}

export interface RewardValuesDto {
  silver_exact: ExactDecimal;
  gold_exact: ExactDecimal;
  diamond_exact: ExactDecimal;
}

export interface PayrollCycleDto {
  cycle_id: string;
  start_date: string;
  end_date: string;
  workday_count: number;
  monthly_salary_exact: ExactDecimal;
  daily_salary_exact: ExactDecimal;
  hourly_salary_exact: ExactDecimal;
}

export interface AppSnapshotDto {
  current_local_time: string;
  work_status: string;
  effective_work_seconds_today: number;
  payroll_cycle: PayrollCycleDto;
  real_payroll: {
    today_real_earned_exact: ExactDecimal;
    cycle_real_earned_exact: ExactDecimal;
  };
  reward_entitlement: {
    today: RewardCountsDto;
    values: RewardValuesDto;
  };
  collected_wallet: {
    today_collected_exact: ExactDecimal;
    cycle_collected_exact: ExactDecimal;
  };
  offline: {
    unclaimed_bag_count: number;
    unclaimed_exact_total: ExactDecimal;
  };
}

export interface OfflineRewardBagDto {
  bag_id: string;
  cycle_id: string;
  period_start: string;
  period_end: string;
  counts: RewardCountsDto;
  exact_value: ExactDecimal;
  created_at: string;
}

export interface CollectionLedgerEntryDto {
  transaction_id: string;
  cycle_id: string;
  source_type: string;
  source_id: string;
  counts: RewardCountsDto;
  exact_value: ExactDecimal;
  created_at: string;
}

export type WalletDisplayMode = "REAL_SALARY" | "COLLECTED_WALLET";

export interface AppSettingsDto {
  wallet_display_mode: WalletDisplayMode;
  sound_enabled: boolean;
  auto_collect_enabled: boolean;
}
