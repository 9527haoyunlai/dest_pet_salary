import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import * as api from "./shared/tauri-api";
import {
  bagFixture,
  calendarFixture,
  configurationFixture,
  settingsFixture,
  snapshotFixture,
  liveRewardFixture,
} from "./test/fixtures";

vi.mock("./features/game/PixiGameScene", () => ({
  PixiGameScene: ({
    liveRewards,
    onCollectLiveReward,
  }: {
    liveRewards: typeof liveRewardFixture[];
    onCollectLiveReward: (eventId: string) => Promise<void>;
  }) => (
    <section aria-label="Mock Pixi lawn">
      {liveRewards.map((event) => (
        <button
          type="button"
          key={event.event_id}
          onClick={() => {
            void onCollectLiveReward(event.event_id).catch(() => undefined);
          }}
        >
          Collect live {event.event_id}
        </button>
      ))}
    </section>
  ),
}));

vi.mock("./shared/tauri-api", () => ({
  claimOfflineRewardBag: vi.fn(),
  collectLiveReward: vi.fn(),
  getAppSettings: vi.fn(),
  getAppSnapshot: vi.fn(),
  getCalendarMonth: vi.fn(),
  getSalaryConfiguration: vi.fn(),
  initializeSalary: vi.fn(),
  listOfflineRewardBags: vi.fn(),
  listPendingLiveRewards: vi.fn(),
  syncLiveRewards: vi.fn(),
  updateAppSettings: vi.fn(),
  updateNextCycleSalary: vi.fn(),
}));

function setDefaultApiResponses() {
  vi.mocked(api.getSalaryConfiguration).mockResolvedValue(configurationFixture);
  vi.mocked(api.getAppSettings).mockResolvedValue(settingsFixture);
  vi.mocked(api.getAppSnapshot).mockResolvedValue(snapshotFixture);
  vi.mocked(api.getCalendarMonth).mockImplementation(async (year, month) =>
    calendarFixture(year, month),
  );
  vi.mocked(api.listOfflineRewardBags).mockResolvedValue([]);
  vi.mocked(api.listPendingLiveRewards).mockResolvedValue([]);
  vi.mocked(api.syncLiveRewards).mockResolvedValue([]);
  vi.mocked(api.updateAppSettings).mockImplementation(async (settings) => settings);
  vi.mocked(api.updateNextCycleSalary).mockResolvedValue(configurationFixture);
  vi.mocked(api.claimOfflineRewardBag).mockResolvedValue({
    transaction_id: "transaction-1",
    cycle_id: bagFixture.cycle_id,
    source_type: "OFFLINE_BAG_CLAIM",
    source_id: bagFixture.bag_id,
    counts: bagFixture.counts,
    exact_value: bagFixture.exact_value,
    created_at: "2026-08-17T10:01:00Z",
  });
  vi.mocked(api.collectLiveReward).mockResolvedValue({
    transaction_id: "live-transaction-1",
    cycle_id: liveRewardFixture.cycle_id,
    source_type: "LIVE_REWARD_COLLECTION",
    source_id: liveRewardFixture.event_id,
    counts: { silver: 1, gold: 0, diamond: 0 },
    exact_value: liveRewardFixture.exact_value,
    created_at: "2026-08-17T02:00:11Z",
  });
}

describe("Salary Garden product UI", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setDefaultApiResponses();
  });

  it("switches wallet presentation without invoking a backend write", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole("heading", { name: "Real-time salary" });
    await user.click(screen.getByRole("button", { name: "Game wallet" }));

    expect(screen.getByRole("heading", { name: "Collected wallet" })).toBeInTheDocument();
    expect(api.updateAppSettings).not.toHaveBeenCalled();
    expect(api.claimOfflineRewardBag).not.toHaveBeenCalled();
  });

  it("passes the exact salary string to the next-cycle command", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByRole("heading", { name: "Real-time salary" });
    await user.click(screen.getByRole("button", { name: "Settings" }));
    const input = screen.getByRole("textbox", { name: "Next cycle salary" });
    await user.clear(input);
    await user.type(input, "13500.50");
    await user.click(screen.getByRole("button", { name: "Apply to next cycle" }));

    await waitFor(() =>
      expect(api.updateNextCycleSalary).toHaveBeenCalledWith("13500.50"),
    );
  });

  it("claims an offline bag through Rust and reloads bags and snapshot", async () => {
    const user = userEvent.setup();
    vi.mocked(api.listOfflineRewardBags)
      .mockResolvedValueOnce([bagFixture])
      .mockResolvedValue([]);
    render(<App />);

    const claimButton = await screen.findByRole("button", { name: "Claim bag" });
    await user.click(claimButton);

    await waitFor(() =>
      expect(api.claimOfflineRewardBag).toHaveBeenCalledWith(bagFixture.bag_id),
    );
    await waitFor(() => expect(screen.queryByText("Offline reward bag")).not.toBeInTheDocument());
    expect(api.listOfflineRewardBags).toHaveBeenCalledTimes(2);
    expect(api.getAppSnapshot).toHaveBeenCalledTimes(2);
  });

  it("shows command errors without a white screen and supports retry", async () => {
    const user = userEvent.setup();
    vi.mocked(api.getSalaryConfiguration)
      .mockRejectedValueOnce(new Error("database unavailable"))
      .mockResolvedValue(configurationFixture);
    render(<App />);

    expect(await screen.findByRole("alert")).toHaveTextContent("database unavailable");
    await user.click(screen.getByRole("button", { name: "Retry" }));

    expect(await screen.findByRole("heading", { name: "Real-time salary" })).toBeInTheDocument();
  });

  it("renders pending live rewards and settles only through the Rust command", async () => {
    const user = userEvent.setup();
    vi.mocked(api.listPendingLiveRewards).mockResolvedValue([liveRewardFixture]);
    vi.mocked(api.syncLiveRewards)
      .mockResolvedValueOnce([liveRewardFixture])
      .mockResolvedValue([]);
    render(<App />);

    const coin = await screen.findByRole("button", {
      name: `Collect live ${liveRewardFixture.event_id}`,
    });
    await user.click(coin);
    await waitFor(() =>
      expect(api.collectLiveReward).toHaveBeenCalledWith(liveRewardFixture.event_id),
    );
    await waitFor(() => expect(coin).not.toBeInTheDocument());
    expect(api.getAppSnapshot).toHaveBeenCalledTimes(2);
  });

  it("keeps a live reward visible when settlement fails", async () => {
    const user = userEvent.setup();
    vi.mocked(api.listPendingLiveRewards).mockResolvedValue([liveRewardFixture]);
    vi.mocked(api.syncLiveRewards).mockResolvedValue([liveRewardFixture]);
    vi.mocked(api.collectLiveReward).mockRejectedValue(new Error("database busy"));
    render(<App />);

    const coin = await screen.findByRole("button", {
      name: `Collect live ${liveRewardFixture.event_id}`,
    });
    await user.click(coin);
    expect(await screen.findByRole("alert")).toHaveTextContent("database busy");
    expect(coin).toBeInTheDocument();
  });
});
