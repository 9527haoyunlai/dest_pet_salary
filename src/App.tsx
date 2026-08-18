import { useCallback, useEffect, useState } from "react";

import { useAppSnapshot } from "./app/hooks/useAppSnapshot";
import { useLiveRewards } from "./app/hooks/useLiveRewards";
import { PvzSkinSurface } from "./app/PvzSkinSurface";
import { pvzUiAssets } from "./assets/pvz-ui";
import { ErrorState } from "./components/ErrorState";
import { LoadingState } from "./components/LoadingState";
import { CalendarPanel } from "./features/calendar/CalendarPanel";
import { Dashboard } from "./features/dashboard/Dashboard";
import { SettingsPanel } from "./features/settings/SettingsPanel";
import {
  claimOfflineRewardBag,
  getAppSettings,
  getSalaryConfiguration,
  initializeSalary,
  listOfflineRewardBags,
  updateAppSettings,
  updateNextCycleSalary,
} from "./shared/tauri-api";
import type {
  AppSettingsDto,
  OfflineRewardBagDto,
  SalaryConfigurationDto,
  WalletDisplayMode,
} from "./shared/types";

type ProductView = "dashboard" | "calendar" | "settings";

function SalarySetup({ onInitialized }: { onInitialized: () => Promise<void> }) {
  const [salary, setSalary] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    setBusy(true);
    setError(null);
    try {
      await initializeSalary(salary);
      await onInitialized();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  };

  return (
    <main className="onboarding-shell">
      <section className="onboarding-card">
        <p className="eyebrow">Welcome to</p>
        <h1>Salary Garden</h1>
        <p>Set your monthly salary to grow an exact, local-first payroll view.</p>
        {error ? <p className="inline-error" role="alert">{error}</p> : null}
        <label className="field-label">
          <span>Monthly salary</span>
          <input
            type="text"
            inputMode="decimal"
            value={salary}
            onChange={(event) => setSalary(event.target.value)}
            placeholder="22000.00"
            autoFocus
          />
        </label>
        <button
          type="button"
          className="primary-button"
          disabled={busy || salary.trim() === ""}
          onClick={() => void submit()}
        >
          {busy ? "Creating payroll cycle…" : "Start Salary Garden"}
        </button>
        <small>Amounts remain exact Decimal strings in the Rust backend.</small>
      </section>
    </main>
  );
}

function AppContent() {
  const [configuration, setConfiguration] =
    useState<SalaryConfigurationDto | null>(null);
  const [settings, setSettings] = useState<AppSettingsDto | null>(null);
  const [bags, setBags] = useState<OfflineRewardBagDto[]>([]);
  const [walletMode, setWalletMode] =
    useState<WalletDisplayMode>("REAL_SALARY");
  const [view, setView] = useState<ProductView>("dashboard");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [claimingBagId, setClaimingBagId] = useState<string | null>(null);

  const initialized = configuration?.is_initialized === true;
  const snapshotState = useAppSnapshot(initialized);
  const refreshAfterLiveSettlement = useCallback(async () => {
    await snapshotState.refresh();
  }, [snapshotState.refresh]);
  const liveRewardsState = useLiveRewards(initialized, refreshAfterLiveSettlement);

  const loadApplication = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const nextConfiguration = await getSalaryConfiguration();
      setConfiguration(nextConfiguration);
      if (nextConfiguration.is_initialized) {
        const [nextSettings, nextBags] = await Promise.all([
          getAppSettings(),
          listOfflineRewardBags(),
        ]);
        setSettings(nextSettings);
        setWalletMode(nextSettings.wallet_display_mode);
        setBags(nextBags);
      } else {
        setSettings(null);
        setBags([]);
      }
    } catch (reason) {
      setError(String(reason));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadApplication();
  }, [loadApplication]);

  useEffect(() => {
    if (!initialized) return;
    const reconcileAfterResume = () => {
      if (document.visibilityState !== "visible") return;
      void liveRewardsState.sync().then(async () => {
        setBags(await listOfflineRewardBags());
      });
    };
    window.addEventListener("focus", reconcileAfterResume);
    document.addEventListener("visibilitychange", reconcileAfterResume);
    return () => {
      window.removeEventListener("focus", reconcileAfterResume);
      document.removeEventListener("visibilitychange", reconcileAfterResume);
    };
  }, [initialized, liveRewardsState.sync]);

  const claimBag = async (bagId: string) => {
    setClaimingBagId(bagId);
    setActionError(null);
    try {
      await claimOfflineRewardBag(bagId);
      setBags(await listOfflineRewardBags());
      await snapshotState.refresh();
    } catch (reason) {
      setActionError(String(reason));
    } finally {
      setClaimingBagId(null);
    }
  };

  const updateSalary = async (exact: string) => {
    setConfiguration(await updateNextCycleSalary(exact));
  };

  const saveSettings = async (nextSettings: AppSettingsDto) => {
    setSettings(await updateAppSettings(nextSettings));
  };

  if (loading) return <LoadingState />;
  if (error) return <ErrorState message={error} onRetry={() => void loadApplication()} />;
  if (!configuration?.is_initialized) {
    return <SalarySetup onInitialized={loadApplication} />;
  }
  if (snapshotState.loading && !snapshotState.snapshot) return <LoadingState />;
  if (snapshotState.error && !snapshotState.snapshot) {
    return (
      <ErrorState
        message={snapshotState.error}
        onRetry={() => void snapshotState.refresh()}
      />
    );
  }
  if (!snapshotState.snapshot || !settings) return <LoadingState />;

  const today = snapshotState.snapshot.current_local_time.slice(0, 10);

  return (
    <main className="product-shell">
      <header className="app-header">
        <div className="brand-lockup">
          <span className="brand-mark" aria-hidden="true">
            <img src={pvzUiAssets.rewards.sun} alt="" />
          </span>
          <span>
            <strong>Salary Garden</strong>
            <small>Phase 5C · Reward Presentation</small>
          </span>
        </div>
        <nav className="primary-navigation" aria-label="Primary navigation">
          {(["dashboard", "calendar", "settings"] as const).map((item) => (
            <button
              type="button"
              key={item}
              aria-current={view === item ? "page" : undefined}
              onClick={() => setView(item)}
            >
              {item[0].toUpperCase() + item.slice(1)}
            </button>
          ))}
        </nav>
        <button
          type="button"
          className="refresh-button"
          onClick={() => {
            void Promise.all([
              snapshotState.refresh(),
              liveRewardsState.sync(),
              listOfflineRewardBags().then(setBags),
            ]);
          }}
          aria-label="Refresh salary snapshot"
        >
          Refresh
        </button>
      </header>

      {actionError || snapshotState.error || liveRewardsState.error ? (
        <div className="error-banner" role="alert">
          <span>{actionError ?? snapshotState.error ?? liveRewardsState.error}</span>
          <button type="button" onClick={() => setActionError(null)}>Dismiss</button>
        </div>
      ) : null}

      <div className="product-content">
        {view === "dashboard" ? (
          <Dashboard
            snapshot={snapshotState.snapshot}
            bags={bags}
            walletMode={walletMode}
            claimingBagId={claimingBagId}
            onWalletModeChange={setWalletMode}
            onClaimBag={(bagId) => void claimBag(bagId)}
            liveRewards={liveRewardsState.events}
            autoCollectEnabled={settings.auto_collect_enabled}
            onCollectLiveReward={liveRewardsState.collect}
          />
        ) : null}
        {view === "calendar" ? (
          <CalendarPanel configuration={configuration} today={today} />
        ) : null}
        {view === "settings" ? (
          <SettingsPanel
            configuration={configuration}
            settings={settings}
            walletMode={walletMode}
            onWalletModeChange={setWalletMode}
            onUpdateNextSalary={updateSalary}
            onUpdateSettings={saveSettings}
          />
        ) : null}
      </div>
    </main>
  );
}

export default function App() {
  return (
    <PvzSkinSurface>
      <AppContent />
    </PvzSkinSurface>
  );
}
