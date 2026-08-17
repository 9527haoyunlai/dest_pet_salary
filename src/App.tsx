import { useCallback, useEffect, useState } from "react";

import {
  claimOfflineRewardBag,
  getCalendarMonth,
  getAppSnapshot,
  getSalaryConfiguration,
  initializeSalary,
  listOfflineRewardBags,
  updateNextCycleSalary,
} from "./shared/tauri-api";
import type {
  AppSnapshotDto,
  CalendarMonthDto,
  OfflineRewardBagDto,
  SalaryConfigurationDto,
} from "./shared/types";

export default function App() {
  const [snapshot, setSnapshot] = useState<AppSnapshotDto | null>(null);
  const [bags, setBags] = useState<OfflineRewardBagDto[]>([]);
  const [salaryConfiguration, setSalaryConfiguration] =
    useState<SalaryConfigurationDto | null>(null);
  const [calendar, setCalendar] = useState<CalendarMonthDto | null>(null);
  const [initialSalary, setInitialSalary] = useState("");
  const [nextCycleSalary, setNextCycleSalary] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const configuration = await getSalaryConfiguration();
      const nextCalendar = await getCalendarMonth(
        configuration.current_year,
        configuration.current_month,
      );
      setSalaryConfiguration(configuration);
      setCalendar(nextCalendar);
      setNextCycleSalary(configuration.next_cycle_salary_exact ?? "");

      if (configuration.is_initialized) {
        const [nextSnapshot, nextBags] = await Promise.all([
          getAppSnapshot(),
          listOfflineRewardBags(),
        ]);
        setSnapshot(nextSnapshot);
        setBags(nextBags);
      } else {
        setSnapshot(null);
        setBags([]);
      }
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const claimFirstBag = async () => {
    const bag = bags[0];
    if (!bag) return;

    setBusy(true);
    setError(null);
    try {
      await claimOfflineRewardBag(bag.bag_id);
      await refresh();
    } catch (reason) {
      setError(String(reason));
      setBusy(false);
    }
  };

  const initialize = async () => {
    setBusy(true);
    setError(null);
    try {
      await initializeSalary(initialSalary);
      setInitialSalary("");
      await refresh();
    } catch (reason) {
      setError(String(reason));
      setBusy(false);
    }
  };

  const applyNextCycleSalary = async () => {
    setBusy(true);
    setError(null);
    try {
      await updateNextCycleSalary(nextCycleSalary);
      await refresh();
    } catch (reason) {
      setError(String(reason));
      setBusy(false);
    }
  };

  return (
    <main className="app-shell">
      <section className="status-card" aria-labelledby="app-title">
        <p className="phase-label">Phase 4A · Salary Config / Calendar API</p>
        <h1 id="app-title">Salary Garden Debug</h1>

        <section className="debug-section" aria-labelledby="salary-config-title">
          <h2 id="salary-config-title">Salary Configuration</h2>
          {salaryConfiguration?.is_initialized &&
          salaryConfiguration.current_cycle ? (
            <>
              <p>
                Current Monthly Salary: ¥
                {salaryConfiguration.current_cycle.monthly_salary_exact}
              </p>
              <label className="debug-input">
                <span>Next Cycle Salary</span>
                <input
                  type="text"
                  inputMode="decimal"
                  value={nextCycleSalary}
                  onChange={(event) => setNextCycleSalary(event.target.value)}
                  placeholder="22000.00"
                />
              </label>
              <button
                type="button"
                onClick={() => void applyNextCycleSalary()}
                disabled={busy || nextCycleSalary.trim() === ""}
              >
                Apply Next Cycle
              </button>
            </>
          ) : (
            <>
              <label className="debug-input">
                <span>Monthly Salary</span>
                <input
                  type="text"
                  inputMode="decimal"
                  value={initialSalary}
                  onChange={(event) => setInitialSalary(event.target.value)}
                  placeholder="22000.00"
                />
              </label>
              <button
                type="button"
                onClick={() => void initialize()}
                disabled={busy || initialSalary.trim() === ""}
              >
                Initialize Salary
              </button>
            </>
          )}
        </section>

        {snapshot ? (
          <dl className="debug-grid">
            <dt>Local Time</dt>
            <dd>{snapshot.current_local_time}</dd>
            <dt>Work Status</dt>
            <dd>{snapshot.work_status}</dd>
            <dt>Payroll Cycle</dt>
            <dd>{snapshot.payroll_cycle.cycle_id}</dd>
            <dt>Today Real Earned</dt>
            <dd>¥{snapshot.real_payroll.today_real_earned_exact}</dd>
            <dt>Cycle Real Earned</dt>
            <dd>¥{snapshot.real_payroll.cycle_real_earned_exact}</dd>
            <dt>Silver</dt>
            <dd>
              {snapshot.reward_entitlement.today.silver} × ¥
              {snapshot.reward_entitlement.values.silver_exact}
            </dd>
            <dt>Gold</dt>
            <dd>
              {snapshot.reward_entitlement.today.gold} × ¥
              {snapshot.reward_entitlement.values.gold_exact}
            </dd>
            <dt>Diamond</dt>
            <dd>
              {snapshot.reward_entitlement.today.diamond} × ¥
              {snapshot.reward_entitlement.values.diamond_exact}
            </dd>
            <dt>Collected Today</dt>
            <dd>¥{snapshot.collected_wallet.today_collected_exact}</dd>
            <dt>Collected Cycle</dt>
            <dd>¥{snapshot.collected_wallet.cycle_collected_exact}</dd>
            <dt>Offline Bags</dt>
            <dd>
              {snapshot.offline.unclaimed_bag_count} / ¥
              {snapshot.offline.unclaimed_exact_total}
            </dd>
          </dl>
        ) : (
          <p className="loading">
            Initialize salary to enable the authoritative Rust snapshot.
          </p>
        )}

        {calendar ? (
          <section className="debug-section" aria-labelledby="calendar-debug-title">
            <h2 id="calendar-debug-title">Calendar Debug</h2>
            <p>
              {calendar.year}-{String(calendar.month).padStart(2, "0")} · Workdays: {" "}
              {calendar.workday_count}
            </p>
            <p>
              Cycle: {calendar.cycle_start} → {calendar.cycle_end} · Payday: {" "}
              {calendar.payday}
            </p>
            <ul className="calendar-days">
              {calendar.days.map((day) => (
                <li key={day.date}>
                  {day.date} · {day.weekday} · {" "}
                  {day.is_workday
                    ? "WORKDAY"
                    : day.is_holiday
                      ? `HOLIDAY${day.holiday_name ? ` (${day.holiday_name})` : ""}`
                      : "WEEKEND"}
                </li>
              ))}
            </ul>
          </section>
        ) : null}

        {error ? <p className="error-message">{error}</p> : null}

        <div className="debug-actions">
          <button type="button" onClick={() => void refresh()} disabled={busy}>
            Refresh
          </button>
          {bags.length > 0 ? (
            <button type="button" onClick={() => void claimFirstBag()} disabled={busy}>
              Claim Bag
            </button>
          ) : null}
        </div>
      </section>
    </main>
  );
}
