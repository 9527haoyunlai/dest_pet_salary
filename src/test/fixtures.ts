import type {
  AppSettingsDto,
  AppSnapshotDto,
  CalendarMonthDto,
  OfflineRewardBagDto,
  LiveRewardEventDto,
  SalaryConfigurationDto,
} from "../shared/types";

export const snapshotFixture: AppSnapshotDto = {
  current_local_time: "2026-08-17T10:00:00+08:00",
  work_status: "WORKING_AM",
  effective_work_seconds_today: 4800,
  payroll_cycle: {
    cycle_id: "2026-08",
    start_date: "2026-08-03",
    end_date: "2026-08-31",
    workday_count: 21,
    monthly_salary_exact: "12000",
    daily_salary_exact: "571.42857142857142857142857143",
    hourly_salary_exact: "81.632653061224489795918367347",
  },
  real_payroll: {
    today_real_earned_exact: "155.49132947976878612716763008",
    cycle_real_earned_exact: "6155.4913294797687861271676301",
  },
  reward_entitlement: {
    today: { silver: 400, gold: 79, diamond: 1 },
    values: {
      silver_exact: "0.080505574",
      gold_exact: "0.483033444",
      diamond_exact: "28.98200664",
    },
  },
  collected_wallet: {
    today_collected_exact: "45.25",
    cycle_collected_exact: "908.75",
  },
  offline: {
    unclaimed_bag_count: 1,
    unclaimed_exact_total: "19.875",
  },
};

export const settingsFixture: AppSettingsDto = {
  wallet_display_mode: "REAL_SALARY",
  sound_enabled: true,
  auto_collect_enabled: true,
};

export const configurationFixture: SalaryConfigurationDto = {
  is_initialized: true,
  timezone: "Asia/Shanghai",
  current_year: 2026,
  current_month: 8,
  current_cycle: {
    cycle_id: "2026-08",
    start_date: "2026-08-03",
    end_date: "2026-08-31",
    workday_count: 21,
    monthly_salary_exact: "12000",
    daily_salary_exact: "571.42857142857142857142857143",
    hourly_salary_exact: "81.632653061224489795918367347",
    per_second_salary_exact: "0.0226757369614512471655328798",
    silver_value_exact: "0.080505574",
    gold_value_exact: "0.483033444",
    diamond_value_exact: "28.98200664",
  },
  next_cycle_salary_exact: "12000",
};

export const bagFixture: OfflineRewardBagDto = {
  bag_id: "bag-1",
  cycle_id: "2026-08",
  period_start: "2026-08-17T09:00:00+08:00",
  period_end: "2026-08-17T10:00:00+08:00",
  counts: { silver: 25, gold: 4, diamond: 0 },
  exact_value: "19.875",
  created_at: "2026-08-17T10:00:00Z",
};

export const liveRewardFixture: LiveRewardEventDto = {
  event_id: "2026-08:2026-08-17:481",
  cycle_id: "2026-08",
  work_date: "2026-08-17",
  effective_second_boundary: 4810,
  event_index: 481,
  reward_type: "SILVER",
  status: "PENDING",
  exact_value: "0.080505574",
  created_at: "2026-08-17T02:00:10Z",
};

export function calendarFixture(year = 2026, month = 8): CalendarMonthDto {
  const isAugust = year === 2026 && month === 8;
  return {
    year,
    month,
    timezone: "Asia/Shanghai",
    cycle_id: `${year}-${String(month).padStart(2, "0")}`,
    cycle_start: isAugust ? "2026-08-03" : "2026-09-01",
    cycle_end: isAugust ? "2026-08-31" : "2026-09-30",
    workday_count: isAugust ? 21 : 22,
    payday: isAugust ? "2026-08-31" : "2026-09-30",
    days: isAugust
      ? [
          {
            date: "2026-08-01",
            weekday: "SATURDAY",
            is_workday: false,
            is_weekend: true,
            is_holiday: false,
            holiday_name: null,
          },
          {
            date: "2026-08-03",
            weekday: "MONDAY",
            is_workday: true,
            is_weekend: false,
            is_holiday: false,
            holiday_name: null,
          },
          {
            date: "2026-08-17",
            weekday: "MONDAY",
            is_workday: true,
            is_weekend: false,
            is_holiday: false,
            holiday_name: null,
          },
        ]
      : [
          {
            date: "2026-09-01",
            weekday: "TUESDAY",
            is_workday: true,
            is_weekend: false,
            is_holiday: false,
            holiday_name: null,
          },
        ],
  };
}
