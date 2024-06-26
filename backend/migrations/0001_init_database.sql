SET TIME ZONE 'UTC';
CREATE EXTENSION IF NOT EXISTS pgcrypto;

DROP TYPE IF EXISTS ACCSTATUS;
DROP TYPE IF EXISTS PERMISSIONS;
CREATE TYPE ACCSTATUS AS ENUM ('active', 'disabled');
CREATE TYPE PERMISSIONS AS ENUM ('admin', 'moderator', 'photographer');

DROP TABLE IF EXISTS users;
CREATE TABLE users (
    user_id BIGSERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password TEXT NOT NULL,
    account_status ACCSTATUS NOT NULL,
    user_perms PERMISSIONS NOT NULL,
    created TIMESTAMPTZ NOT NULL,
    last_login TIMESTAMPTZ
);

DROP TABLE IF EXISTS sessions;
CREATE TABLE sessions (
    cntr BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES users (user_id),
    session_id TEXT UNIQUE NOT NULL,
    created TIMESTAMPTZ NOT NULL,
    expires TIMESTAMPTZ NOT NULL
)