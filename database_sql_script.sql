
-- TEMP MIGRATION SCRIPT 
-- This script initialises the database for cards, decks, and views

CREATE TABLE IF NOT EXISTS cards (
    id uuid PRIMARY KEY NOT NULL,
    front_text TEXT,
    back_text TEXT,
    deck_id INTEGER,
    FOREIGN KEY (deck_id) REFERENCES decks(id),
    created DATETIME,
    modified DATETIME,
);

CREATE TABLE IF NOT EXISTS views (
    id uuid PRIMARY KEY NOT NULL,
    card_id INTEGER,
    FOREIGN KEY (card_id) REFERENCES cards(id),
    result BOOLEAN,
    time DATETIME,
);

CREATE TABLE IF NOT EXISTS decks (
    id uuid INTEGER PRIMARY KEY NOT NULL,
    name TEXT,
    created DATETIME,
    modified DATETIME,
);
