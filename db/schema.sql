-- Stocked schema for SingleStore (Helios). Every household-scoped row carries household_id,
-- so Firedrill drill sessions can run against cloned households without touching demo data.
-- Statements are separated by ';' on their own logical line; core/src/db.rs applies them in order.

CREATE TABLE IF NOT EXISTS members (
  household_id VARCHAR(64) NOT NULL,
  id VARCHAR(64) NOT NULL,
  name VARCHAR(64) NOT NULL,
  role VARCHAR(16) NOT NULL,            -- 'approver' | 'member'
  SHARD KEY (household_id),
  SORT KEY (household_id, id)
);

CREATE TABLE IF NOT EXISTS items (
  household_id VARCHAR(64) NOT NULL,
  id VARCHAR(64) NOT NULL,
  name VARCHAR(128) NOT NULL,
  category VARCHAR(32) NOT NULL,
  unit VARCHAR(32) NOT NULL,
  is_shared_consumable BOOL NOT NULL,
  correction VARCHAR(16) NULL,          -- 'out' | 'plenty' (one-tap corrections)
  correction_at DATETIME NULL,
  corrected_by VARCHAR(64) NULL,
  SHARD KEY (household_id),
  SORT KEY (household_id, id)
);

CREATE TABLE IF NOT EXISTS receipts (
  household_id VARCHAR(64) NOT NULL,
  id VARCHAR(64) NOT NULL,
  source VARCHAR(16) NOT NULL,          -- 'airbyte' | 'seeded'
  source_ref VARCHAR(255) NULL,         -- e.g. Gmail message id
  member_id VARCHAR(64) NOT NULL,
  store VARCHAR(64) NOT NULL,
  received_at DATETIME NOT NULL,
  subject VARCHAR(255) NOT NULL,
  raw_text TEXT NOT NULL,
  SHARD KEY (household_id),
  SORT KEY (household_id, received_at)
);

CREATE TABLE IF NOT EXISTS purchases (
  household_id VARCHAR(64) NOT NULL,
  id VARCHAR(64) NOT NULL,
  receipt_id VARCHAR(64) NOT NULL,
  item_id VARCHAR(64) NOT NULL,
  qty INT NOT NULL,
  unit_price_cents BIGINT NOT NULL,
  purchased_at DATETIME NOT NULL,
  evidence_span TEXT NOT NULL,          -- exact text from the receipt
  verified BOOL NOT NULL,               -- span found verbatim in raw_text
  extractor VARCHAR(64) NOT NULL,       -- model id or 'seed'
  SHARD KEY (household_id),
  SORT KEY (household_id, item_id, purchased_at)
);

CREATE TABLE IF NOT EXISTS calendar_events (
  household_id VARCHAR(64) NOT NULL,
  id VARCHAR(64) NOT NULL,
  title VARCHAR(255) NOT NULL,
  starts_at DATETIME NOT NULL,
  guests INT NOT NULL,
  kind VARCHAR(24) NOT NULL DEFAULT 'party',   -- party | houseguests | dinner | away
  nights INT NOT NULL DEFAULT 0,
  host_member_id VARCHAR(64) NOT NULL,
  source VARCHAR(16) NOT NULL,          -- 'airbyte' | 'simulated' | 'seeded'
  source_ref VARCHAR(255) NULL,
  SHARD KEY (household_id),
  SORT KEY (household_id, starts_at)
);

CREATE TABLE IF NOT EXISTS products (
  id VARCHAR(64) NOT NULL,
  item_id VARCHAR(64) NOT NULL,
  store VARCHAR(64) NOT NULL,
  channel VARCHAR(16) NOT NULL,         -- 'online' | 'instore'
  title VARCHAR(255) NOT NULL,
  description TEXT NOT NULL,
  price_cents BIGINT NOT NULL,
  delivery_days INT NOT NULL,           -- 0 for in-store pickup
  cutoff_hour INT NOT NULL,             -- online: order by this hour for delivery_days; instore: opening hour
  embedding VECTOR(1024) NULL,
  SHARD KEY (id),
  SORT KEY (item_id),
  FULLTEXT USING VERSION 2 products_ft (title, description)
);

CREATE TABLE IF NOT EXISTS plans (
  household_id VARCHAR(64) NOT NULL,
  id VARCHAR(64) NOT NULL,
  version INT NOT NULL,
  kind VARCHAR(16) NOT NULL,            -- 'event' | 'restock' | 'request'
  title VARCHAR(255) NOT NULL,
  trigger_ref VARCHAR(64) NULL,
  requested_by VARCHAR(64) NULL,
  basket JSON NOT NULL,
  checks JSON NOT NULL,
  total_cents BIGINT NOT NULL,
  status VARCHAR(24) NOT NULL,          -- held | auto_approved | approved | refused | skipped | ordered | payment_failed
  agent_version VARCHAR(8) NOT NULL,
  created_at DATETIME NOT NULL,
  decided_by VARCHAR(64) NULL,
  decided_at DATETIME NULL,
  decision_note TEXT NULL,
  SHARD KEY (household_id),
  SORT KEY (household_id, created_at)
);

-- Unique keys on columnstore must contain the shard key: shard on the idempotency key itself.
CREATE TABLE IF NOT EXISTS orders (
  idempotency_key VARCHAR(128) NOT NULL,
  household_id VARCHAR(64) NOT NULL,
  id VARCHAR(64) NOT NULL,
  plan_id VARCHAR(64) NOT NULL,
  total_cents BIGINT NOT NULL,
  paid_by VARCHAR(64) NOT NULL,
  payment_provider VARCHAR(32) NOT NULL,
  payment_ref VARCHAR(255) NOT NULL,
  splits JSON NOT NULL,
  test_mode BOOL NOT NULL,
  created_at DATETIME NOT NULL,
  SHARD KEY (idempotency_key),
  UNIQUE KEY (idempotency_key) USING HASH,
  SORT KEY (household_id, created_at)
);

CREATE TABLE IF NOT EXISTS agent_events (
  household_id VARCHAR(64) NOT NULL,
  id VARCHAR(64) NOT NULL,
  ts DATETIME(6) NOT NULL,
  kind VARCHAR(48) NOT NULL,
  subject_type VARCHAR(24) NOT NULL,    -- item | plan | order | receipt | event | request | system
  subject_id VARCHAR(64) NOT NULL,
  actor VARCHAR(64) NOT NULL,           -- 'agent' | member id | 'firedrill'
  summary TEXT NOT NULL,
  data JSON NOT NULL,
  model VARCHAR(128) NULL,
  latency_us BIGINT NOT NULL,
  cost_credits DOUBLE NOT NULL,
  provenance VARCHAR(16) NOT NULL,      -- observed | inferred | computed
  forwarded BOOL NOT NULL DEFAULT 0,    -- sent to Amplitude
  SHARD KEY (household_id),
  SORT KEY (household_id, ts)
);

-- Identity checks: who is the human behind an approval (Persona inquiry, or a labelled simulation).
CREATE TABLE IF NOT EXISTS verifications (
  household_id VARCHAR(64) NOT NULL,
  id VARCHAR(64) NOT NULL,
  member_id VARCHAR(64) NOT NULL,
  plan_id VARCHAR(64) NULL,
  provider VARCHAR(24) NOT NULL,        -- persona | simulated
  inquiry_id VARCHAR(128) NULL,
  status VARCHAR(16) NOT NULL,          -- pending | approved | declined
  url TEXT NULL,
  created_at DATETIME NOT NULL,
  verified_at DATETIME NULL,
  SHARD KEY (household_id),
  SORT KEY (household_id, created_at)
);

-- Outings: shared plans whose cost depends on headcount (dinner reservations, cabs, day trips).
CREATE TABLE IF NOT EXISTS outings (
  household_id VARCHAR(64) NOT NULL,
  id VARCHAR(64) NOT NULL,
  version INT NOT NULL,
  title VARCHAR(255) NOT NULL,
  kind VARCHAR(24) NOT NULL,            -- dinner | trip
  venue VARCHAR(255) NOT NULL,
  starts_at DATETIME NOT NULL,
  organizer VARCHAR(64) NOT NULL,
  pricing JSON NOT NULL,                -- per_head, cab, cab_xl, transit fares and extra minutes
  invitees JSON NOT NULL,               -- each person's agent's answer
  quote JSON NOT NULL,                  -- the chosen option at the current headcount
  booking JSON NULL,
  status VARCHAR(16) NOT NULL,          -- planned | booked
  source VARCHAR(16) NOT NULL,
  source_ref VARCHAR(255) NULL,
  created_at DATETIME NOT NULL,
  SHARD KEY (household_id),
  SORT KEY (household_id, starts_at)
);
