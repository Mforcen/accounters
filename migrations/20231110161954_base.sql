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
    tx_date DATETIME,
    category INTEGER,
    amount INTEGER,
    accumulated INTEGER DEFAULT 0,
    tx_order INTEGER DEFAULT 0,
    hash TEXT,
    FOREIGN KEY (account) REFERENCES accounts(account_id),
    FOREIGN KEY (category) REFERENCES categories(category_id)
);

CREATE TRIGGER tx_insert AFTER INSERT ON transactions
BEGIN
    UPDATE transactions 
    SET accumulated=old.acc+NEW.amount 
    FROM (
        SELECT COALESCE(max(accumulated), 0) AS acc
        FROM transactions
        WHERE date <= NEW.date
        ORDER BY tx_order DESC
        LIMIT 1
    ) AS old 
    WHERE id=NEW.id;

    UPDATE transactions
    SET tx_order=old.tx_order+1 FROM (
        SELECT COALESCE(max(tx_order), 0) as tx_order
        FROM tx WHERE date=NEW.date
    ) AS old
    WHERE id=NEW.id;

    UPDATE transactions SET accumulated=calc.acc+NEW.accumulated FROM (
        SELECT tx.id, (
            SUM(amount) OVER (
                ORDER BY date, tx_order
                ROWS BETWEEN
                UNBOUNDED PRECEDING
                AND CURRENT ROW
            )
        ) acc
        FROM transactions tx
        WHERE date > NEW.date OR id=NEW.id;
    )
    WHERE transactions.id=calc.id;
END;
CREATE INDEX idx_transactions_ts ON transactions(account, tx_date);
CREATE INDEX idx_transactions_hash ON transactions(hash); 
