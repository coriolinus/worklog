-- events tables and indices
CREATE TABLE evt_type (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE events (
    id INTEGER PRIMARY KEY NOT NULL,
    evt_type INTEGER NOT NULL REFERENCES evt_type(id),
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    message TEXT NOT NULL DEFAULT ''
);

CREATE INDEX events_timestamps ON events (timestamp);

-- default event types
INSERT INTO evt_type (name) VALUES ('START');
INSERT INTO evt_type (name) VALUES ('STOP');
