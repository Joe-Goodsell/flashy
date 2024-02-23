-- /migrations
-- This script initialises the database for cards, decks, and views

CREATE TABLE IF NOT EXISTS decks (
    id uuid NOT NULL,
    PRIMARY KEY(id),
    name TEXT UNIQUE,
    created TIMESTAMP WITH TIME ZONE,
    modified TIMESTAMP WITH TIME ZONE
);

CREATE TABLE IF NOT EXISTS cards (
    id uuid NOT NULL,
    PRIMARY KEY(id),
    front_text TEXT,
    back_text TEXT,
    deck_id uuid,
    FOREIGN KEY (deck_id) REFERENCES decks(id),
    created TIMESTAMP WITH TIME ZONE,
    modified TIMESTAMP WITH TIME ZONE
);

CREATE TABLE IF NOT EXISTS views (
    id uuid NOT NULL,
    PRIMARY KEY(id),
    card_id uuid,
    FOREIGN KEY (card_id) REFERENCES cards(id),
    result BOOLEAN,
    time TIMESTAMP WITH TIME ZONE
);

