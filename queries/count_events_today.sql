SELECT count(*) FROM events
WHERE events.timestamp >= ? AND events.timestamp < ?
;
