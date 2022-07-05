SELECT id, evt_type, timestamp, message
FROM events
WHERE
    timestamp >= ?
    AND timestamp < ?
ORDER BY timestamp ASC
;
