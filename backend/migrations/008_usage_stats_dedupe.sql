DELETE us1 FROM usage_stats us1
JOIN usage_stats us2
  ON us1.user_id = us2.user_id
 AND us1.stat_key = us2.stat_key
 AND (
   us1.updated_at < us2.updated_at
   OR (us1.updated_at = us2.updated_at AND us1.id < us2.id)
 );

DROP INDEX uq_usage_stats_user_key ON usage_stats;
CREATE UNIQUE INDEX uq_usage_stats_user_key ON usage_stats(user_id, stat_key);
