CREATE TABLE users
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    first_name TEXT NOT NULL,
    last_name  TEXT NOT NULL,
    email      TEXT NOT NULL UNIQUE,
    role       TEXT CHECK( role in ('user','admin') ) NOT NULL DEFAULT 'user',
    disabled   TEXT,
    last_login INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE test (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rowdata TEXT[] NOT NULL
);
