import type { AppSnapshotDto } from "../../shared/types";

const STATUS_LABELS: Record<string, string> = {
  NON_WORKDAY: "Rest day",
  BEFORE_WORK: "Work has not started",
  WORKING_AM: "Morning work",
  LUNCH_BREAK: "Lunch break",
  WORKING_PM: "Afternoon work",
  AFTER_WORK: "Workday complete",
};

interface WorkStatusProps {
  snapshot: AppSnapshotDto;
}

export function WorkStatus({ snapshot }: WorkStatusProps) {
  return (
    <section className="work-status" aria-label="Current work status">
      <span className={`status-dot status-${snapshot.work_status.toLowerCase()}`} />
      <span>
        <small>Work status</small>
        <strong>{STATUS_LABELS[snapshot.work_status] ?? snapshot.work_status}</strong>
      </span>
      <time dateTime={snapshot.current_local_time}>{snapshot.current_local_time}</time>
    </section>
  );
}
