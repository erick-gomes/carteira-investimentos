-- Add migration script here
CREATE TABLE IF NOT EXISTS assets (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ticker TEXT,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    quantity_raw BIGINT NOT NULL,
    average_price_cents BIGINT NOT NULL,
    currency TEXT NOT NULL,
    last_acquisition_date DATE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_user_ticker UNIQUE (user_id, ticker)
);