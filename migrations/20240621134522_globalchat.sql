-- Add migration script here
CREATE TABLE globalchat (
    name TEXT NOT NULL PRIMARY KEY,
    created_by BIGINT NOT NULL
);

CREATE TABLE globalchat_channels (
    id BIGINT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    FOREIGN KEY (name) REFERENCES globalchat(name) ON DELETE CASCADE
)