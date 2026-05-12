CREATE TABLE IF NOT EXISTS stock_master (
    stock_code TEXT PRIMARY KEY,
    stock_name TEXT NOT NULL,
    market TEXT,
    sector TEXT,
    is_preferred INTEGER NOT NULL DEFAULT 0,
    is_etf INTEGER NOT NULL DEFAULT 0,
    is_etn INTEGER NOT NULL DEFAULT 0,
    is_spac INTEGER NOT NULL DEFAULT 0,
    is_trading_halted INTEGER NOT NULL DEFAULT 0,
    is_administrative_issue INTEGER NOT NULL DEFAULT 0,
    is_warning INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS stock_ohlcv_daily (
    trade_date TEXT NOT NULL,
    stock_code TEXT NOT NULL,
    open_price TEXT NOT NULL,
    high_price TEXT NOT NULL,
    low_price TEXT NOT NULL,
    close_price TEXT NOT NULL,
    volume INTEGER NOT NULL,
    trade_amount INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (trade_date, stock_code)
);

CREATE INDEX IF NOT EXISTS ix_ohlcv_stock_date
ON stock_ohlcv_daily (stock_code, trade_date);

CREATE INDEX IF NOT EXISTS ix_ohlcv_trade_amount
ON stock_ohlcv_daily (trade_date, trade_amount);

CREATE TABLE IF NOT EXISTS stock_investor_flow_daily (
    trade_date TEXT NOT NULL,
    stock_code TEXT NOT NULL,
    institution_net_buy_amt INTEGER NOT NULL DEFAULT 0,
    foreign_net_buy_amt INTEGER NOT NULL DEFAULT 0,
    individual_net_buy_amt INTEGER NOT NULL DEFAULT 0,
    institution_net_buy_qty INTEGER,
    foreign_net_buy_qty INTEGER,
    individual_net_buy_qty INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (trade_date, stock_code)
);

CREATE INDEX IF NOT EXISTS ix_flow_stock_date
ON stock_investor_flow_daily (stock_code, trade_date);

CREATE INDEX IF NOT EXISTS ix_flow_inst_amt
ON stock_investor_flow_daily (trade_date, institution_net_buy_amt);

CREATE INDEX IF NOT EXISTS ix_flow_foreign_amt
ON stock_investor_flow_daily (trade_date, foreign_net_buy_amt);

CREATE TABLE IF NOT EXISTS stock_financial_ratio (
    stock_code TEXT PRIMARY KEY,
    debt_ratio TEXT,
    operating_profit INTEGER,
    quarterly_operating_profit INTEGER,
    capital_impairment INTEGER,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS stock_market_metric_daily (
    trade_date TEXT NOT NULL,
    stock_code TEXT NOT NULL,
    market_cap INTEGER,
    credit_balance INTEGER,
    short_selling_amount INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (trade_date, stock_code)
);

CREATE TABLE IF NOT EXISTS trade_candidate_daily (
    trade_date TEXT NOT NULL,
    stock_code TEXT NOT NULL,
    stock_name TEXT NOT NULL,
    supply_score TEXT NOT NULL,
    liquidity_score TEXT NOT NULL,
    trend_score TEXT NOT NULL,
    risk_score TEXT NOT NULL,
    final_score TEXT NOT NULL,
    inst_buy_days INTEGER NOT NULL,
    foreign_buy_days INTEGER NOT NULL,
    inst_net_buy_amt_5d INTEGER NOT NULL,
    foreign_net_buy_amt_5d INTEGER NOT NULL,
    total_net_buy_amt_5d INTEGER NOT NULL,
    market_cap INTEGER,
    avg_trade_amount_20d INTEGER,
    prev_close TEXT NOT NULL,
    prev_high TEXT NOT NULL,
    prev_low TEXT NOT NULL,
    ma5 TEXT,
    ma20 TEXT,
    breakout_price TEXT,
    pullback_price TEXT,
    stop_loss_price TEXT,
    target_price TEXT,
    strategy_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'READY',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (trade_date, stock_code, strategy_type)
);

CREATE INDEX IF NOT EXISTS ix_candidate_status_score
ON trade_candidate_daily (trade_date, status, final_score);

CREATE INDEX IF NOT EXISTS ix_candidate_stock_code
ON trade_candidate_daily (stock_code);

CREATE TABLE IF NOT EXISTS trade_signal (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    signal_date TEXT NOT NULL,
    signal_time TEXT NOT NULL,
    stock_code TEXT NOT NULL,
    stock_name TEXT NOT NULL,
    strategy_type TEXT NOT NULL,
    signal_price TEXT NOT NULL,
    final_score TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS ix_signal_date_stock
ON trade_signal (signal_date, stock_code);

CREATE TABLE IF NOT EXISTS trade_order (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trade_date TEXT NOT NULL,
    stock_code TEXT NOT NULL,
    stock_name TEXT NOT NULL,
    side TEXT NOT NULL,
    order_type TEXT NOT NULL,
    order_price TEXT,
    order_qty INTEGER NOT NULL,
    kis_order_no TEXT,
    status TEXT NOT NULL DEFAULT 'REQUESTED',
    reason TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT
);

CREATE INDEX IF NOT EXISTS ix_order_trade_date_stock
ON trade_order (trade_date, stock_code);

CREATE INDEX IF NOT EXISTS ix_order_status
ON trade_order (status);

CREATE TABLE IF NOT EXISTS bot_run_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command TEXT NOT NULL,
    run_date TEXT,
    status TEXT NOT NULL,
    message TEXT,
    started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    finished_at TEXT
);
