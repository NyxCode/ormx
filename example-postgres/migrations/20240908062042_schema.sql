CREATE TYPE user_role AS ENUM ('user', 'admin');
CREATE TYPE account_type AS ENUM ('legacy', 'normal');
CREATE TYPE user_group AS ENUM ('local', 'global', 'other');

CREATE TABLE users
(
    id         SERIAL PRIMARY KEY,
    first_name VARCHAR(128) NOT NULL,
    last_name  VARCHAR(128) NOT NULL,
    email      VARCHAR(128) NOT NULL UNIQUE,
    role       user_role    NOT NULL,
    type       account_type,
    "group"    user_group   NOT NULL DEFAULT 'local',
    disabled   TEXT,
    last_login TIMESTAMP DEFAULT NULL
);

CREATE TABLE test
(
    id   SERIAL PRIMARY KEY,
    rows TEXT[] NOT NULL
);