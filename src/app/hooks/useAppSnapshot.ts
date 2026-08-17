import { useCallback, useEffect, useRef, useState } from "react";

import { getAppSnapshot } from "../../shared/tauri-api";
import type { AppSnapshotDto } from "../../shared/types";

const DEFAULT_REFRESH_INTERVAL_MS = 750;

export interface AppSnapshotState {
  snapshot: AppSnapshotDto | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export function useAppSnapshot(
  enabled: boolean,
  refreshIntervalMs = DEFAULT_REFRESH_INTERVAL_MS,
): AppSnapshotState {
  const [snapshot, setSnapshot] = useState<AppSnapshotDto | null>(null);
  const [loading, setLoading] = useState(enabled);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(true);

  const refresh = useCallback(async () => {
    if (!enabled) return;
    try {
      const nextSnapshot = await getAppSnapshot();
      if (!mounted.current) return;
      setSnapshot(nextSnapshot);
      setError(null);
    } catch (reason) {
      if (mounted.current) setError(String(reason));
    } finally {
      if (mounted.current) setLoading(false);
    }
  }, [enabled]);

  useEffect(() => {
    mounted.current = true;
    if (!enabled) {
      setSnapshot(null);
      setLoading(false);
      setError(null);
      return () => {
        mounted.current = false;
      };
    }

    setLoading(true);
    void refresh();
    const interval = window.setInterval(() => void refresh(), refreshIntervalMs);
    return () => {
      mounted.current = false;
      window.clearInterval(interval);
    };
  }, [enabled, refresh, refreshIntervalMs]);

  return { snapshot, loading, error, refresh };
}
