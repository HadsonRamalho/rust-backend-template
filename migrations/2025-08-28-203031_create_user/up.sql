-- Your SQL goes here

CREATE TABLE IF NOT EXISTS users(
    id UUID PRIMARY KEY,
    public_id INTEGER NOT NULL UNIQUE,
    name VARCHAR(128) NOT NULL,
    email VARCHAR(64) NOT NULL UNIQUE,
    document VARCHAR(32) NOT NULL UNIQUE,
    password VARCHAR(128) NOT NULL,
    birthdate DATE NOT NULL,
    login_type VARCHAR(16) NOT NULL,
    user_type VARCHAR(16) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    create_date TIMESTAMP NOT NULL,
    update_date TIMESTAMP NOT NULL,
    deletion_date TIMESTAMP
)
