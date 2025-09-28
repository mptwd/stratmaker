CREATE TABLE IF NOT EXISTS strategies (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  title VARCHAR(255) NOT NULL,
  trade_type INTEGER, -- ? maybe if its option trading or whatever TODO: check
  content JSONB NOT NULL, -- json representing the strategy
  create_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  update_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
)
