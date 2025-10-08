-- setup cron for running pgpartman maintenance

SELECT cron.schedule('pgpartman-maintenance', '*/5 * * * *',
    'CALL run_maintenance_proc()');
