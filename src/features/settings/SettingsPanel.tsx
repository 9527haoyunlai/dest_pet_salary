import { useEffect, useState } from "react";

import { formatCycleRange, formatExactCurrency } from "../../app/format";
import type {
  AppSettingsDto,
  SalaryConfigurationDto,
  WalletDisplayMode,
} from "../../shared/types";

interface SettingsPanelProps {
  configuration: SalaryConfigurationDto;
  settings: AppSettingsDto;
  walletMode: WalletDisplayMode;
  onWalletModeChange: (mode: WalletDisplayMode) => void;
  onUpdateNextSalary: (exact: string) => Promise<void>;
  onUpdateSettings: (settings: AppSettingsDto) => Promise<void>;
}

export function SettingsPanel({
  configuration,
  settings,
  walletMode,
  onWalletModeChange,
  onUpdateNextSalary,
  onUpdateSettings,
}: SettingsPanelProps) {
  const [nextSalary, setNextSalary] = useState(
    configuration.next_cycle_salary_exact ?? "",
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setNextSalary(configuration.next_cycle_salary_exact ?? "");
  }, [configuration.next_cycle_salary_exact]);

  const updateSalary = async () => {
    setBusy(true);
    setError(null);
    try {
      await onUpdateNextSalary(nextSalary);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  };

  const updateBooleanSetting = async (
    key: "sound_enabled" | "auto_collect_enabled",
    checked: boolean,
  ) => {
    setBusy(true);
    setError(null);
    try {
      await onUpdateSettings({ ...settings, [key]: checked });
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  };

  const cycle = configuration.current_cycle;

  return (
    <section className="product-panel settings-panel" aria-labelledby="settings-title">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Preferences</p>
          <h2 id="settings-title">Settings</h2>
        </div>
        <span className="timezone-label">{configuration.timezone}</span>
      </div>

      {error ? <p className="inline-error" role="alert">{error}</p> : null}

      <div className="settings-grid">
        <section className="settings-section" aria-labelledby="salary-settings-title">
          <h3 id="salary-settings-title">Salary</h3>
          {cycle ? (
            <dl className="settings-facts">
              <div>
                <dt>Current monthly salary</dt>
                <dd>{formatExactCurrency(cycle.monthly_salary_exact)}</dd>
              </div>
              <div>
                <dt>Current payroll cycle</dt>
                <dd>{formatCycleRange(cycle.start_date, cycle.end_date)}</dd>
              </div>
              <div>
                <dt>Workdays</dt>
                <dd>{cycle.workday_count}</dd>
              </div>
              <div>
                <dt>Daily / Hourly</dt>
                <dd>
                  {formatExactCurrency(cycle.daily_salary_exact)} / {" "}
                  {formatExactCurrency(cycle.hourly_salary_exact)}
                </dd>
              </div>
            </dl>
          ) : null}

          <label className="field-label">
            <span>Next cycle salary</span>
            <input
              type="text"
              inputMode="decimal"
              value={nextSalary}
              onChange={(event) => setNextSalary(event.target.value)}
              placeholder="22000.00"
            />
          </label>
          <button
            type="button"
            className="primary-button"
            disabled={busy || nextSalary.trim() === ""}
            onClick={() => void updateSalary()}
          >
            Apply to next cycle
          </button>
        </section>

        <section className="settings-section" aria-labelledby="display-settings-title">
          <h3 id="display-settings-title">Display & game</h3>
          <div className="setting-row">
            <span>
              <strong>Wallet display mode</strong>
              <small>Changes presentation only</small>
            </span>
            <div className="segmented-control compact" aria-label="Settings wallet mode">
              <button
                type="button"
                aria-pressed={walletMode === "REAL_SALARY"}
                onClick={() => onWalletModeChange("REAL_SALARY")}
              >
                Salary
              </button>
              <button
                type="button"
                aria-pressed={walletMode === "COLLECTED_WALLET"}
                onClick={() => onWalletModeChange("COLLECTED_WALLET")}
              >
                Wallet
              </button>
            </div>
          </div>

          <label className="setting-row">
            <span>
              <strong>Auto collect</strong>
              <small>Magnet-shroom collects live rewards after a short pause</small>
            </span>
            <input
              type="checkbox"
              checked={settings.auto_collect_enabled}
              disabled={busy}
              onChange={(event) =>
                void updateBooleanSetting("auto_collect_enabled", event.target.checked)
              }
            />
          </label>
          <label className="setting-row">
            <span>
              <strong>Sound enabled</strong>
              <small>Stored preference; no sound is played in this phase</small>
            </span>
            <input
              type="checkbox"
              checked={settings.sound_enabled}
              disabled={busy}
              onChange={(event) =>
                void updateBooleanSetting("sound_enabled", event.target.checked)
              }
            />
          </label>
          <div className="setting-row is-later">
            <span>
              <strong>Always on top</strong>
              <small>Later · Phase 8 desktop integration</small>
            </span>
          </div>
          <div className="setting-row is-later">
            <span>
              <strong>Launch at startup</strong>
              <small>Later · Phase 8 desktop integration</small>
            </span>
          </div>
        </section>
      </div>
    </section>
  );
}
