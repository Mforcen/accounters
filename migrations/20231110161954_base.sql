-- Add migration script here

CREATE TABLE IF NOT EXISTS users(
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT,
    pass TEXT,
    UNIQUE(username)
);

CREATE TABLE IF NOT EXISTS accounts(
    account_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user INTEGER,
    account_name TEXT,
    FOREIGN KEY (user) REFERENCES users(user_id)
);

CREATE TABLE IF NOT EXISTS account_snapshot(
    account INTEGER,
    datestamp DATE,
    amount INT,
    FOREIGN KEY (account) REFERENCES accounts(account_id),
    PRIMARY KEY (account, datestamp)
);

CREATE TABLE IF NOT EXISTS categories (
    category_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS rules(
    rule_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user INTEGER,
    regex TEXT,
    category INTEGER,
    FOREIGN KEY (user) REFERENCES users(user_id)
    FOREIGN KEY (category) REFERENCES categories(category_id)
);

CREATE TABLE IF NOT EXISTS transactions (
    transaction_id INTEGER PRIMARY KEY AUTOINCREMENT,
    account INTEGER,
    description TEXT,
    transaction_timestamp DATETIME,
    category INTEGER,
    amount INTEGER,
    hash TEXT,
    FOREIGN KEY (account) REFERENCES accounts(account_id),
    FOREIGN KEY (category) REFERENCES categories(category_id)
);

CREATE INDEX idx_transactions_ts ON transactions(account, transaction_timestamp);
CREATE INDEX idx_transactions_hash ON transactions(hash);
