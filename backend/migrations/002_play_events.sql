CREATE TABLE IF NOT EXISTS play_events (
  id CHAR(36) PRIMARY KEY,
  track_id CHAR(36) NULL,
  source VARCHAR(32) NOT NULL DEFAULT 'local',
  source_id VARCHAR(128) NULL,
  title VARCHAR(255) NULL,
  artist VARCHAR(255) NULL,
  played_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX idx_play_events_date (played_at),
  INDEX idx_play_events_track (track_id),
  CONSTRAINT fk_play_events_track FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE SET NULL
);
