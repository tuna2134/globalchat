-- Add migration script here
CREATE TABLE globalchat (
    name TEXT NOT NULL PRIMARY KEY,
    channels BIGINT[] NOT NULL DEFAULT '{}'
);