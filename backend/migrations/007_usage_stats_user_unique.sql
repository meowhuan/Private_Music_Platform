ALTER TABLE usage_stats
  DROP INDEX stat_key;

CREATE UNIQUE INDEX uq_usage_stats_user_key ON usage_stats(user_id, stat_key);
