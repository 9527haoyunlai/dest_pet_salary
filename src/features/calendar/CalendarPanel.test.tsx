import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { getCalendarMonth } from "../../shared/tauri-api";
import { calendarFixture, configurationFixture } from "../../test/fixtures";
import { CalendarPanel, navigateCalendarMonth } from "./CalendarPanel";

vi.mock("../../shared/tauri-api", () => ({
  getCalendarMonth: vi.fn(),
}));

describe("CalendarPanel", () => {
  beforeEach(() => {
    vi.mocked(getCalendarMonth).mockImplementation(async (year, month) =>
      calendarFixture(year, month),
    );
  });

  it("navigates months through the Rust calendar API", async () => {
    const user = userEvent.setup();
    render(<CalendarPanel configuration={configurationFixture} today="2026-08-17" />);

    await screen.findByRole("heading", { name: "2026-08" });
    await user.click(screen.getByRole("button", { name: "Next month" }));

    await waitFor(() => expect(getCalendarMonth).toHaveBeenLastCalledWith(2026, 9));
    expect(screen.getByRole("heading", { name: "2026-09" })).toBeInTheDocument();
  });

  it("handles year boundaries without deriving workday rules", () => {
    expect(navigateCalendarMonth({ year: 2026, month: 12 }, 1)).toEqual({
      year: 2027,
      month: 1,
    });
    expect(navigateCalendarMonth({ year: 2026, month: 1 }, -1)).toEqual({
      year: 2025,
      month: 12,
    });
  });
});
