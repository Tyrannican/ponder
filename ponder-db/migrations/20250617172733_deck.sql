-- Add migration script here
create table if not exists deck (
    id integer primary key,
    name text not null,
    format_id integer not null,
    foreign key (format_id) references format(id)
);

create table if not exists deck_entry(
    deck_id integer not null,
    card_id text not null,
    entry_type text not null check (entry_type in ('main', 'sideboard', 'commander')),
    quantity integer not null default 1,
    primary key (deck_id, card_id, entry_type),
    foreign key (deck_id) references deck(id),
    foreign key (card_id) references card(card_id) 
);

create index if not exists idx_deck_entry_deck on deck_entry(deck_id);
create index if not exists idx_deck_entry_card on deck_entry(card_id);
create index if not exists idx_deck_entry_type on deck_entry(entry_type);
