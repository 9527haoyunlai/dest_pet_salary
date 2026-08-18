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
  const collectionsInFlight = useRef(new Map<string, Promise<void>>());

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
    (eventId: string): Promise<void> => {
      const existing = collectionsInFlight.current.get(eventId);
      if (existing) return existing;

      const request = (async () => {
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
        try {
          const pending = await listPendingLiveRewards();
          const stillPending = pending.some((event) => event.event_id === eventId);
          if (mounted.current) setEvents(pending);
          if (!stillPending) {
            // Another collector won the race. Rust is authoritative, so treat the
            // absent pending event as settled and refresh rather than showing it again.
            if (mounted.current) setError(null);
            await onSettled();
            await sync();
            return;
          }
        } catch {
          // Preserve the original collection error; a failed confirmation cannot
          // safely classify the event as already settled.
        }
        if (mounted.current) setError(String(reason));
        throw reason;
      }
      })().finally(() => {
        collectionsInFlight.current.delete(eventId);
      });
      collectionsInFlight.current.set(eventId, request);
      return request;
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
        collectionsInFlight.current.clear();
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
      collectionsInFlight.current.clear();
    };
  }, [enabled, sync, syncIntervalMs]);

  return { events, error, sync, collect };
}
