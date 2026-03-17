ALTER TABLE tracks
  ADD COLUMN user_id CHAR(36) NULL;
ALTER TABLE playlists
  ADD COLUMN user_id CHAR(36) NULL;
ALTER TABLE playlist_tracks
  ADD COLUMN user_id CHAR(36) NULL;
ALTER TABLE queue
  ADD COLUMN user_id CHAR(36) NULL;
ALTER TABLE now_playing
  ADD COLUMN user_id CHAR(36) NULL;
ALTER TABLE usage_stats
  ADD COLUMN user_id CHAR(36) NULL;
ALTER TABLE devices
  ADD COLUMN user_id CHAR(36) NULL;
ALTER TABLE play_events
  ADD COLUMN user_id CHAR(36) NULL;

CREATE INDEX idx_tracks_user ON tracks(user_id);
CREATE INDEX idx_playlists_user ON playlists(user_id);
CREATE INDEX idx_playlist_tracks_user ON playlist_tracks(user_id);
CREATE INDEX idx_queue_user ON queue(user_id);
CREATE INDEX idx_now_playing_user ON now_playing(user_id);
CREATE INDEX idx_usage_stats_user ON usage_stats(user_id);
CREATE INDEX idx_devices_user ON devices(user_id);
CREATE INDEX idx_play_events_user ON play_events(user_id);
