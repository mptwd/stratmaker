-- Fast PostgreSQL Job Queue Schema
-- Uses SKIP LOCKED for concurrent workers without contention

CREATE TYPE job_status AS ENUM ('pending', 'running', 'done', 'failed', 'cancelled');

--- This is a general job table, it is not fixed to backtests jobs (if i ever need another type of job)
CREATE TABLE jobs (
    id BIGSERIAL PRIMARY KEY,

    -- Job identification
    job_type VARCHAR(100) NOT NULL,

    -- Payload stored as MessagePack for efficiency
    payload BYTEA NOT NULL,
 
    -- Priority (higher = more important)
    priority INTEGER NOT NULL DEFAULT 0,

    -- Status tracking
    status job_status NOT NULL DEFAULT 'pending',

    -- Retry logic
    max_retries INTEGER NOT NULL DEFAULT 3,
    retry_count INTEGER NOT NULL DEFAULT 0,

    -- Timing
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    scheduled_for TIMESTAMPTZ NOT NULL DEFAULT NOW(), -- For delayed jobs
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    -- Error tracking
    last_error TEXT,

    -- Worker tracking (optional, for debugging)
    worker_id VARCHAR(100),

    -- Metadata
    timeout_seconds INTEGER NOT NULL DEFAULT 300
);

-- Critical indexes for performance
-- Composite index for the main queue query (MOST IMPORTANT)
CREATE INDEX idx_jobs_queue ON jobs (status, scheduled_for, priority DESC, id)
    WHERE status IN ('pending', 'failed');

-- Index for monitoring
CREATE INDEX idx_jobs_status ON jobs (status);
CREATE INDEX idx_jobs_job_type ON jobs (job_type);
CREATE INDEX idx_jobs_created_at ON jobs (created_at DESC);

-- For cleanup of old completed jobs
CREATE INDEX idx_jobs_cleanup ON jobs (status, completed_at)
    WHERE status IN ('done', 'cancelled');

-- Partial index for stuck jobs (running too long)
CREATE INDEX idx_jobs_stuck ON jobs (status, started_at)
    WHERE status = 'running';

-- Function to enqueue a job
CREATE OR REPLACE FUNCTION enqueue_job(
    p_job_type VARCHAR,
    p_payload BYTEA,
    p_priority INTEGER DEFAULT 0,
    p_max_retries INTEGER DEFAULT 3,
    p_delay_seconds INTEGER DEFAULT 0,
    p_timeout_seconds INTEGER DEFAULT 300
)
RETURNS BIGINT AS $$
DECLARE
    job_id BIGINT;
BEGIN
    INSERT INTO jobs (
        job_type,
        payload,
        priority,
        max_retries,
        scheduled_for,
        timeout_seconds
    )
    VALUES (
        p_job_type,
        p_payload,
        p_priority,
        p_max_retries,
        NOW() + (p_delay_seconds || ' seconds')::INTERVAL,
        p_timeout_seconds
    )
    RETURNING id INTO job_id;
    
    RETURN job_id;
END;
$$ LANGUAGE plpgsql;

-- Function to dequeue a job (SKIP LOCKED for concurrency)
-- This is the CORE function - optimized for speed
CREATE OR REPLACE FUNCTION dequeue_job(
    p_worker_id VARCHAR,
    p_job_types VARCHAR[] DEFAULT NULL -- Filter by job types if provided
)
RETURNS TABLE (
    id BIGINT,
    job_type VARCHAR,
    payload BYTEA,
    retry_count INTEGER,
    timeout_seconds INTEGER
) AS $$
DECLARE
    job_record RECORD;
BEGIN
    -- Lock and claim the next available job
    SELECT j.id, j.job_type, j.payload, j.retry_count, j.timeout_seconds
    INTO job_record
    FROM jobs j
    WHERE j.status IN ('pending', 'failed')
        AND j.scheduled_for <= NOW()
        AND j.retry_count < j.max_retries
        AND (p_job_types IS NULL OR j.job_type = ANY(p_job_types))
    ORDER BY j.priority DESC, j.id
    LIMIT 1
    FOR UPDATE SKIP LOCKED;
    
    IF NOT FOUND THEN
        RETURN;
    END IF;
    
    -- Update job status
    UPDATE jobs
    SET 
        status = 'running',
        started_at = NOW(),
        worker_id = p_worker_id,
        retry_count = retry_count + 1
    WHERE jobs.id = job_record.id;
    
    -- Return job details
    RETURN QUERY
    SELECT 
        job_record.id,
        job_record.job_type,
        job_record.payload,
        job_record.retry_count,
        job_record.timeout_seconds;
END;
$$ LANGUAGE plpgsql;

-- Function to mark job as completed
CREATE OR REPLACE FUNCTION complete_job(p_job_id BIGINT)
RETURNS BOOLEAN AS $$
DECLARE
    rows_updated INTEGER;
BEGIN
    UPDATE jobs
    SET 
        status = 'done',
        completed_at = NOW()
    WHERE id = p_job_id
        AND status = 'running';
    
    GET DIAGNOSTICS rows_updated = ROW_COUNT;
    RETURN rows_updated > 0;
END;
$$ LANGUAGE plpgsql;

-- Function to mark job as failed
CREATE OR REPLACE FUNCTION fail_job(
    p_job_id BIGINT,
    p_error_message TEXT
)
RETURNS BOOLEAN AS $$
DECLARE
    rows_updated INTEGER;
    current_retry_count INTEGER;
    max_retry_count INTEGER;
BEGIN
    -- Get current retry info
    SELECT retry_count, max_retries
    INTO current_retry_count, max_retry_count
    FROM jobs
    WHERE id = p_job_id;
    
    -- If we've exhausted retries, mark as failed permanently
    IF current_retry_count >= max_retry_count THEN
        UPDATE jobs
        SET 
            status = 'failed',
            completed_at = NOW(),
            last_error = p_error_message
        WHERE id = p_job_id
            AND status = 'running';
    ELSE
        -- Otherwise, mark as pending for retry
        UPDATE jobs
        SET 
            status = 'pending',
            last_error = p_error_message,
            -- Exponential backoff: 2^retry_count minutes
            scheduled_for = NOW() + (POWER(2, retry_count) || ' minutes')::INTERVAL
        WHERE id = p_job_id
            AND status = 'running';
    END IF;
    
    GET DIAGNOSTICS rows_updated = ROW_COUNT;
    RETURN rows_updated > 0;
END;
$$ LANGUAGE plpgsql;

-- Function to recover stuck jobs (run periodically)
-- This could be usefull in case a worker crashes, and did not update the job back
CREATE OR REPLACE FUNCTION recover_stuck_jobs()
RETURNS INTEGER AS $$
DECLARE
    recovered_count INTEGER;
BEGIN
    UPDATE jobs
    SET 
        status = 'pending',
        started_at = NULL,
        worker_id = NULL,
        last_error = 'Job timed out and was recovered'
    WHERE status = 'running'
        AND started_at < NOW() - (timeout_seconds || ' seconds')::INTERVAL;
    
    GET DIAGNOSTICS recovered_count = ROW_COUNT;
    RETURN recovered_count;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up old completed jobs (clean jobs completed more than p_days_old days ago)
CREATE OR REPLACE FUNCTION cleanup_old_jobs(p_days_old INTEGER DEFAULT 7)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM jobs
    WHERE status IN ('done', 'cancelled')
        AND completed_at < NOW() - (p_days_old || ' days')::INTERVAL;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- View for monitoring queue health
CREATE OR REPLACE VIEW queue_stats AS
SELECT
    job_type,
    status,
    COUNT(*) as count,
    AVG(EXTRACT(EPOCH FROM (completed_at - created_at))) as avg_duration_seconds,
    MAX(retry_count) as max_retries_used
FROM jobs
WHERE created_at > NOW() - INTERVAL '24 hours'
GROUP BY job_type, status
ORDER BY job_type, status;

-- View for stuck jobs
CREATE OR REPLACE VIEW stuck_jobs AS
SELECT
    id,
    job_type,
    worker_id,
    started_at,
    EXTRACT(EPOCH FROM (NOW() - started_at)) as stuck_for_seconds,
    timeout_seconds
FROM jobs
WHERE status = 'running'
    AND started_at < NOW() - (timeout_seconds || ' seconds')::INTERVAL
ORDER BY started_at;

-- Maintenance: Schedule these to run periodically (using pg_cron or external scheduler)
-- SELECT recover_stuck_jobs(); -- Every 5 minutes
-- SELECT cleanup_old_jobs(7);  -- Daily
