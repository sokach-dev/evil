-- Add up migration script here

CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account VARCHAR(255) NOT NULL UNIQUE, -- account address
    created_at INTEGER NOT NULL, -- created at
    deleted INTEGER DEFAULT 0 -- deleted flag , 1 is deleted
);