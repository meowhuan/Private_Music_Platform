ALTER TABLE devices
  ADD COLUMN device_token VARCHAR(128) NULL,
  ADD COLUMN last_sync_at TIMESTAMP NULL,
  ADD COLUMN cache_size_mb INT NULL,
  ADD COLUMN synced_tracks INT NULL,
  ADD COLUMN playing_title VARCHAR(255) NULL,
  ADD COLUMN playing_artist VARCHAR(255) NULL,
  ADD COLUMN playback_progress INT NULL;
