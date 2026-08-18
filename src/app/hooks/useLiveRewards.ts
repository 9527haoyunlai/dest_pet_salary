import { useCallback, useEffect, useRef, useState } from "react";

import {
  collectLiveReward,
  listPendingLiveRewards,
  syncLiveRewards,
} from "../../shared/tauri-api";
import type { LiveRewardEventDto } from "../../shared/types";

const DEFAULT_SYNC_INTERVAL_MS = 1_000;

export interface LiveRewardsState {
  events: LiveRewardEventDto[];
  error: string | null;
  sync: () => Promise<void>;
  collect: (eventId: string) => Promise<void>;
}

export function useLiveRewards(
  enabled: boolean,
  onSettled: () => Promise<void>,
  syncIntervalMs = DEFAULT_SYNC_INTERVAL_MS,
): LiveRewardsState {
  const [events, setEvents] = useState<LiveRewardEventDto[]>([]);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(true);
  const syncing = useRef(false);

  const sync = useCallback(async () => {
    if (!enabled || syncing.current) return;
    syncing.current = true;
    try {
      const pending = await syncLiveRewards();
      if (mounted.current) {
        setEvents(pending);
        setError(null);
      }
    } catch (reason) {
      if (mounted.current) setError(String(reason));
    } finally {
      syncing.current = false;
    }
  }, [enabled]);

  const collect = useCallback(
    async (eventId: string) => {
      try {
        await collectLiveReward(eventId);
        if (mounted.current) {
          // Removal follows Rust transaction success; no wallet arithmetic happens here.
          setEvents((current) => current.filter((event) => event.event_id !== eventId));
          setError(null);
        }
        await onSettled();
        await sync();
      } catch (reason) {
        if (mounted.current) setError(String(reason));
        throw reason;
      }
    },
    [onSettled, sync],
  );

  useEffect(() => {
    mounted.current = true;
    if (!enabled) {
      setEvents([]);
      setError(null);
      return () => {
        mounted.current = false;
      };
    }

    void listPendingLiveRewards()
      .then((pending) => {
        if (mounted.current) setEvents(pending);
      })
      .catch((reason: unknown) => {
        if (mounted.current) setError(String(reason));
      });
    void sync();
    const interval = window.setInterval(() => void sync(), syncIntervalMs);
    return () => {
      mounted.current = false;
      window.clearInterval(interval);
    };
  }, [enabled, sync, syncIntervalMs]);

  return { events, error, sync, collect };
}
