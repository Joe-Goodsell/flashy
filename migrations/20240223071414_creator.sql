-- /migrations
-- This script initialises the database for cards, decks, and views

CREATE TABLE IF NOT EXISTS decks (
    id UUID NOT NULL,
    PRIMARY KEY(id),
    name TEXT UNIQUE NOT NULL,
    created TIMESTAMPTZ,
    modified TIMESTAMPTZ 
);

CREATE TABLE IF NOT EXISTS cards (
    id UUID NOT NULL,
    PRIMARY KEY(id),
    front_text TEXT,
    back_text TEXT,
    deck_id UUID,
    FOREIGN KEY (deck_id) REFERENCES decks(id),
    created TIMESTAMPTZ,
    modified TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS views (
    id UUID NOT NULL,
    PRIMARY KEY(id),
    card_id UUID,
    FOREIGN KEY (card_id) REFERENCES cards(id),
    result BOOLEAN,
    time TIMESTAMPTZ
);

