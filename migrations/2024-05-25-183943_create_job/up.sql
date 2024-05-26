-- Your SQL goes here
CREATE TABLE jobs (
    id SERIAL PRIMARY KEY,
    schedule VARCHAR NOT NULL,
    next_run TIMESTAMP WITH TIME ZONE NOT NULL
);

INSERT INTO jobs (schedule, next_run)
VALUES (
    '*/5 * * * * *',  -- Cron expression for every 10 seconds
    NOW()                -- Set the initial next_run time to the current time
);