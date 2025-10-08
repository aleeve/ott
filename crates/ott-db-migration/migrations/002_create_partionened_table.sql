-- Create the partitioned table

CREATE TABLE vectors (
    id BIGSERIAL,
    uri VARCHAR NOT NULL,
    vector vector,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);
