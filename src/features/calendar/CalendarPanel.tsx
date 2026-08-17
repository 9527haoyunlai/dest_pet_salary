import { useCallback, useEffect, useState } from "react";

import { formatCycleRange, formatExactCurrency } from "../../app/format";
import { getCalendarMonth } from "../../shared/tauri-api";
import type {
  CalendarMonthDto,
  SalaryConfigurationDto,
} from "../../shared/types";

const WEEKDAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const WEEKDAY_INDEX: Record<string, number> = {
  MONDAY: 0,
  TUESDAY: 1,
  WEDNESDAY: 2,
  THURSDAY: 3,
  FRIDAY: 4,
  SATURDAY: 5,
  SUNDAY: 6,
};

export interface CalendarPage {
  year: number;
  month: number;
}

export function navigateCalendarMonth(
  page: CalendarPage,
  offset: -1 | 1,
): CalendarPage {
  if (page.month === 1 && offset === -1) return { year: page.year - 1, month: 12 };
  if (page.month === 12 && offset === 1) return { year: page.year + 1, month: 1 };
  return { year: page.year, month: page.month + offset };
}

interface CalendarPanelProps {
  configuration: SalaryConfigurationDto;
  today: string;
}

export function CalendarPanel({ configuration, today }: CalendarPanelProps) {
  const [page, setPage] = useState<CalendarPage>({
    year: configuration.current_year,
    month: configuration.current_month,
  });
  const [calendar, setCalendar] = useState<CalendarMonthDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadMonth = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setCalendar(await getCalendarMonth(page.year, page.month));
    } catch (reason) {
      setError(String(reason));
    } finally {
      setLoading(false);
    }
  }, [page.month, page.year]);

  useEffect(() => {
    void loadMonth();
  }, [loadMonth]);

  const currentCycle = configuration.current_cycle;
  const firstWeekday = calendar?.days[0]?.weekday;
  const leadingBlanks = firstWeekday ? (WEEKDAY_INDEX[firstWeekday] ?? 0) : 0;

  return (
    <section className="product-panel calendar-panel" aria-labelledby="calendar-title">
      <div className="section-heading calendar-heading">
        <div>
          <p className="eyebrow">Work calendar</p>
          <h2 id="calendar-title">
            {page.year}-{String(page.month).padStart(2, "0")}
          </h2>
        </div>
        <div className="calendar-navigation">
          <button
            type="button"
            aria-label="Previous month"
            onClick={() => setPage((current) => navigateCalendarMonth(current, -1))}
          >
            ←
          </button>
          <button
            type="button"
            onClick={() =>
              setPage({
                year: configuration.current_year,
                month: configuration.current_month,
              })
            }
          >
            Today
          </button>
          <button
            type="button"
            aria-label="Next month"
            onClick={() => setPage((current) => navigateCalendarMonth(current, 1))}
          >
            →
          </button>
        </div>
      </div>

      {error ? (
        <div className="inline-error" role="alert">
          <span>{error}</span>
          <button type="button" onClick={() => void loadMonth()}>
            Retry
          </button>
        </div>
      ) : null}

      <div className={`calendar-grid${loading ? " is-loading" : ""}`}>
        {WEEKDAYS.map((weekday) => (
          <span className="weekday-label" key={weekday}>
            {weekday}
          </span>
        ))}
        {Array.from({ length: leadingBlanks }, (_, index) => (
          <span className="calendar-blank" key={`blank-${index}`} />
        ))}
        {calendar?.days.map((day) => {
          const isToday = day.date === today;
          const isCurrentCycle = Boolean(
            currentCycle &&
              day.date >= currentCycle.start_date &&
              day.date <= currentCycle.end_date,
          );
          const dayLabel = day.is_holiday
            ? day.holiday_name ?? "Holiday"
            : day.is_weekend
              ? "Weekend"
              : "Workday";
          return (
            <article
              className={[
                "calendar-day",
                day.is_workday ? "is-workday" : "is-offday",
                day.is_weekend ? "is-weekend" : "",
                day.is_holiday ? "is-holiday" : "",
                isToday ? "is-today" : "",
                isCurrentCycle ? "is-current-cycle" : "",
              ]
                .filter(Boolean)
                .join(" ")}
              key={day.date}
              aria-label={`${day.date}, ${day.weekday}, ${dayLabel}`}
            >
              <strong>{day.date.slice(-2)}</strong>
              <small>{dayLabel}</small>
            </article>
          );
        })}
      </div>

      {calendar ? (
        <div className="calendar-summary">
          <article>
            <span>Viewed month cycle</span>
            <strong>{formatCycleRange(calendar.cycle_start, calendar.cycle_end)}</strong>
          </article>
          <article>
            <span>Workdays / Payday</span>
            <strong>
              {calendar.workday_count} / {calendar.payday}
            </strong>
          </article>
          {currentCycle ? (
            <>
              <article>
                <span>Current monthly salary</span>
                <strong>{formatExactCurrency(currentCycle.monthly_salary_exact)}</strong>
              </article>
              <article>
                <span>Daily / Hourly</span>
                <strong>
                  {formatExactCurrency(currentCycle.daily_salary_exact)} / {" "}
                  {formatExactCurrency(currentCycle.hourly_salary_exact)}
                </strong>
              </article>
            </>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}
