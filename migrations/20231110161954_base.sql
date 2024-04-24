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
    FOREIGN KEY (account) REFERENCES accounts(account_id),
    FOREIGN KEY (category) REFERENCES categories(category_id)
);

CREATE TRIGGER tx_insert AFTER INSERT ON transactions
BEGIN
    UPDATE transactions 
    SET accumulated=old.acc+NEW.amount 
    FROM (
        SELECT COALESCE(accumulated, 0) AS acc
        FROM transactions
        WHERE tx_date <= NEW.tx_date
            AND transaction_id <> NEW.transaction_id
            AND account=NEW.account
        ORDER BY tx_date DESC, tx_order DESC
        LIMIT 1
    ) AS old 
    WHERE transaction_id=NEW.transaction_id;

    UPDATE transactions
    SET tx_order=old.tx_order+1 FROM (
        SELECT COALESCE(max(tx_order), 0) as tx_order
        FROM transactions WHERE tx_date=NEW.tx_date
    ) AS old
    WHERE transaction_id=NEW.transaction_id;

    UPDATE transactions SET accumulated=calc.acc+cte_tx.accumulated FROM (
        SELECT tx.transaction_id, (
            SUM(amount) OVER (
                ORDER BY tx_date, tx_order
                ROWS BETWEEN
                UNBOUNDED PRECEDING
                AND CURRENT ROW
            )
        ) acc
        FROM transactions tx
        WHERE tx_date > NEW.tx_date AND account=NEW.account
    ) AS calc, (
        SELECT accumulated
        FROM transactions tx
        WHERE tx.transaction_id=NEW.transaction_id
    ) AS cte_tx
    WHERE transactions.transaction_id=calc.transaction_id;
END;
CREATE INDEX idx_transactions_ts ON transactions(account, tx_date);
