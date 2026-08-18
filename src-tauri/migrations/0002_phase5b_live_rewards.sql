CREATE TABLE live_reward_events (
    event_id TEXT PRIMARY KEY NOT NULL,
    cycle_id TEXT NOT NULL,
    work_date TEXT NOT NULL,
    effective_second_boundary INTEGER NOT NULL CHECK (effective_second_boundary > 0),
    event_index INTEGER NOT NULL CHECK (event_index > 0),
    reward_type TEXT NOT NULL CHECK (reward_type IN ('SILVER', 'GOLD', 'DIAMOND')),
    status TEXT NOT NULL CHECK (status IN ('PENDING', 'COLLECTED', 'PACKAGED')),
    exact_value TEXT NOT NULL,
    created_at TEXT NOT NULL,
    collected_at TEXT,
    packaged_bag_id TEXT,
    FOREIGN KEY (cycle_id) REFERENCES payroll_cycles(cycle_id),
    FOREIGN KEY (packaged_bag_id) REFERENCES offline_reward_bags(bag_id),
    UNIQUE (cycle_id, work_date, event_index),
    CHECK (effective_second_boundary = event_index * 10),
    CHECK (
        (status = 'PENDING' AND collected_at IS NULL AND packaged_bag_id IS NULL) OR
        (status = 'COLLECTED' AND collected_at IS NOT NULL AND packaged_bag_id IS NULL) OR
        (status = 'PACKAGED' AND collected_at IS NULL AND packaged_bag_id IS NOT NULL)
    )
);

CREATE INDEX idx_live_reward_events_cycle_status
    ON live_reward_events(cycle_id, status, work_date, event_index);
CREATE INDEX idx_live_reward_events_packaged_bag
    ON live_reward_events(packaged_bag_id);
