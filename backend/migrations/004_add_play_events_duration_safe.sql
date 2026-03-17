SET @col_exists := (
  SELECT COUNT(*)
  FROM INFORMATION_SCHEMA.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'play_events'
    AND COLUMN_NAME = 'duration_seconds'
);

SET @ddl := IF(@col_exists = 0,
  'ALTER TABLE play_events ADD COLUMN duration_seconds INT NULL',
  'SELECT 1'
);

PREPARE stmt FROM @ddl;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
