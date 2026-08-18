import type { AppSnapshotDto } from "../../shared/types";

const STATUS_LABELS: Record<string, string> = {
  NON_WORKDAY: "今天休息",
  BEFORE_WORK: "今日尚未开始",
  WORKING_AM: "上午工作中",
  LUNCH_BREAK: "午休",
  WORKING_PM: "下午工作中",
  AFTER_WORK: "今日已完成",
};

interface WorkStatusProps {
  snapshot: AppSnapshotDto;
}

export function WorkStatus({ snapshot }: WorkStatusProps) {
  return (
    <section className="work-status" aria-label="Current work status">
      <span className={`status-dot status-${snapshot.work_status.toLowerCase()}`} />
      <span>
        <small>当前状态</small>
        <strong>{STATUS_LABELS[snapshot.work_status] ?? snapshot.work_status}</strong>
      </span>
      <time dateTime={snapshot.current_local_time}>{snapshot.current_local_time}</time>
    </section>
  );
}
