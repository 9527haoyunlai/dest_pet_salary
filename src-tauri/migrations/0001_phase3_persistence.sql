CREATE TABLE payroll_cycles (
    cycle_id TEXT PRIMARY KEY NOT NULL,
    salary_month TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    monthly_salary_exact TEXT NOT NULL,
    workday_count INTEGER NOT NULL CHECK (workday_count > 0),
    daily_pay_exact TEXT NOT NULL,
    hourly_pay_exact TEXT NOT NULL,
    per_second_pay_exact TEXT NOT NULL,
    silver_value_exact TEXT NOT NULL,
    gold_value_exact TEXT NOT NULL,
    diamond_value_exact TEXT NOT NULL,
    timezone TEXT NOT NULL,
    calendar_version TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE daily_reward_state (
    cycle_id TEXT NOT NULL,
    work_date TEXT NOT NULL,
    entitled_silver INTEGER NOT NULL DEFAULT 0 CHECK (entitled_silver >= 0),
    entitled_gold INTEGER NOT NULL DEFAULT 0 CHECK (entitled_gold >= 0),
    entitled_diamond INTEGER NOT NULL DEFAULT 0 CHECK (entitled_diamond >= 0),
    accounted_silver INTEGER NOT NULL DEFAULT 0 CHECK (accounted_silver >= 0),
    accounted_gold INTEGER NOT NULL DEFAULT 0 CHECK (accounted_gold >= 0),
    accounted_diamond INTEGER NOT NULL DEFAULT 0 CHECK (accounted_diamond >= 0),
    collected_silver INTEGER NOT NULL DEFAULT 0 CHECK (collected_silver >= 0),
    collected_gold INTEGER NOT NULL DEFAULT 0 CHECK (collected_gold >= 0),
    collected_diamond INTEGER NOT NULL DEFAULT 0 CHECK (collected_diamond >= 0),
    updated_at TEXT NOT NULL,
    PRIMARY KEY (cycle_id, work_date),
    FOREIGN KEY (cycle_id) REFERENCES payroll_cycles(cycle_id),
    CHECK (accounted_silver <= entitled_silver),
    CHECK (accounted_gold <= entitled_gold),
    CHECK (accounted_diamond <= entitled_diamond),
    CHECK (collected_silver <= accounted_silver),
    CHECK (collected_gold <= accounted_gold),
    CHECK (collected_diamond <= accounted_diamond)
);

CREATE TABLE offline_reward_bags (
    bag_id TEXT PRIMARY KEY NOT NULL,
    cycle_id TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    silver_count INTEGER NOT NULL CHECK (silver_count >= 0),
    gold_count INTEGER NOT NULL CHECK (gold_count >= 0),
    diamond_count INTEGER NOT NULL CHECK (diamond_count >= 0),
    exact_value TEXT NOT NULL,
    created_at TEXT NOT NULL,
    claimed INTEGER NOT NULL DEFAULT 0 CHECK (claimed IN (0, 1)),
    claimed_at TEXT,
    FOREIGN KEY (cycle_id) REFERENCES payroll_cycles(cycle_id),
    CHECK (silver_count + gold_count + diamond_count > 0),
    CHECK ((claimed = 0 AND claimed_at IS NULL) OR (claimed = 1 AND claimed_at IS NOT NULL))
);

CREATE TABLE offline_reward_bag_items (
    bag_id TEXT NOT NULL,
    work_date TEXT NOT NULL,
    silver_count INTEGER NOT NULL CHECK (silver_count >= 0),
    gold_count INTEGER NOT NULL CHECK (gold_count >= 0),
    diamond_count INTEGER NOT NULL CHECK (diamond_count >= 0),
    PRIMARY KEY (bag_id, work_date),
    FOREIGN KEY (bag_id) REFERENCES offline_reward_bags(bag_id) ON DELETE RESTRICT,
    CHECK (silver_count + gold_count + diamond_count > 0)
);

CREATE TABLE collection_ledger (
    transaction_id TEXT PRIMARY KEY NOT NULL,
    cycle_id TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK (source_type IN ('OFFLINE_BAG_CLAIM', 'LIVE_REWARD_COLLECTION')),
    source_id TEXT NOT NULL,
    silver_count INTEGER NOT NULL CHECK (silver_count >= 0),
    gold_count INTEGER NOT NULL CHECK (gold_count >= 0),
    diamond_count INTEGER NOT NULL CHECK (diamond_count >= 0),
    exact_value TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (cycle_id) REFERENCES payroll_cycles(cycle_id),
    UNIQUE (source_type, source_id),
    CHECK (silver_count + gold_count + diamond_count > 0)
);

CREATE TABLE app_settings (
    setting_key TEXT PRIMARY KEY NOT NULL,
    setting_value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_daily_reward_state_cycle_date
    ON daily_reward_state(cycle_id, work_date);
CREATE INDEX idx_offline_reward_bags_cycle_claimed
    ON offline_reward_bags(cycle_id, claimed, created_at);
CREATE INDEX idx_collection_ledger_cycle_created
    ON collection_ledger(cycle_id, created_at);
