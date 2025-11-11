CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE backtest_status AS ENUM ('pending', 'running', 'done', 'failed', 'cancelled');

CREATE TABLE IF NOT EXISTS backtests (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

  -- id of the strategy it is running 
  strategy_id UUID NOT NULL REFERENCES strategies(id) ON DELETE CASCADE,

  -- status of the backtest
  status backtest_status NOT NULL DEFAULT 'pending',

  -- dataset it is backtesting on
  dataset VARCHAR(50) NOT NULL, -- e.g. 'BTCUSDT'
  -- timeframe of the dataset
  timeframe VARCHAR(10) NOT NULL, -- e.g. '1m', '5h', etc.

  -- point in time to start the backtest (must be later than start date of the dataset)
  date_start TIMESTAMPTZ NOT NULL,
  -- point in time to end the backtest (must be earlier than end date of the dataset)
  date_end TIMESTAMPTZ NOT NULL,

  -- Last job id that actually computed, or is computing the backtest.
  job_id BIGSERIAL,

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
