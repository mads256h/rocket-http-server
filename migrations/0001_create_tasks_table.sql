CREATE TABLE Tasks(
  id INTEGER PRIMARY KEY NOT NULL,
  timespan_start INTEGER NOT NULL,
  timespan_end   INTEGER NOT NULL,
  duration       INTEGER NOT NULL,
  effect         REAL    NOT NULL,
  device_id      INTEGER NOT NULL
);
