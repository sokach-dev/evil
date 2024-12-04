-- Add up migration script here

-- create table coins in sqlite
CREATE TABLE coins (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL, -- account id
    token VARCHAR(255) NOT NULL, -- token address
    created_at INTEGER NOT NULL, -- created at
    deleted INTEGER DEFAULT 0 -- deleted flag , 1 is deleted
);