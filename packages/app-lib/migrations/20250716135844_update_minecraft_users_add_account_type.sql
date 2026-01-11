-- [AR] - SQL Migration
ALTER TABLE minecraft_users ADD COLUMN account_type varchar(32) NOT NULL DEFAULT 'unknown';

UPDATE minecraft_users SET account_type = 'microsoft' WHERE access_token != 'null';
UPDATE minecraft_users SET account_type = 'pirate' WHERE access_token == 'null';