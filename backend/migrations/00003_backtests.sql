CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE backtest_status AS ENUM ('pending', 'running', 'done', 'failed', 'cancelled');

CREATE TABLE IF NOT EXISTS backtests (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  -- user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE, -- <- This might not be useful
  strategy_id UUID NOT NULL REFERENCES strategies(id) ON DELETE CASCADE,
  status backtest_status NOT NULL DEFAULT 'pending',
  dataset VARCHAR(50) NOT NULL, -- e.g. 'BTCUSDT'
  timeframe VARCHAR(10) NOT NULL, -- e.g. '1m', '5h', etc.
  date_start TIMESTAMP NOT NULL,
  date_end TIMESTAMP NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

  -- Summary stats
  result_summary JSONB,
  -- net_return FLOAT,
  -- cumulative_return FLOAT,
  -- profit_factor FLOAT,
  -- sharpe_ratio FLOAT,
  -- max_drawdown FLOAT,
  -- trades_count INT,
  -- win_rate FLOAT,

  -- references to detailed results
  trades_url TEXT, -- S3/storage path to JSON/Parquet
  equity_curve_url TEXT,
  positions_path TEXT
);
