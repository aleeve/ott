-- Configure pg_partman for the table

-- Configure pg_partman for the table
SELECT create_parent(
    p_parent_table => 'public.vectors',
    p_control => 'created_at',
    p_interval => '30 minutes',
    p_type => 'range',
    p_premake => 8
);

-- Configure retention policy
UPDATE part_config
SET
    retention_keep_table = false,
    retention = '2 hours',
    infinite_time_partitions = true,
    automatic_maintenance = 'on'
WHERE parent_table = 'vectors';
