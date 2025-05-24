create table if not exists card (
    id text not null primary key,
    object text not null,
    name text not null,
    color_indicator text,
    produced_mana text,
    loyalty integer,
    artist text,
    oracle_id text,
    lang text,
    content_warning boolean,
    converted_mana_cost real,
    image_status text,
    flavor_text text,
    arena_id integer,
    illustration_id text,
    oracle_text text,
    color_identity text,
    rarity text,
    power integer,
    toughness integer,
    set_name text,
    set_id text,
    set_type text,
    set_short text,
    penny_rank text,
    variation boolean,
    mtgo_id integer,
    colors text,
    booster boolean,
    border_color text,
    foil boolean,
    game_changer boolean,
    reprint boolean,
    layout text,
    reserved boolean,
    digital boolean,
    keywords text,
    mana_cost text,
    mtgo boolean,
    arena boolean,
    paper boolean,
    promo boolean
);

create table if not exists image_type (
    id integer primary key,
    name text not null unique
);


create table if not exists images (
    card_id text not null,
    image_type_id integer not null,
    uri text not null,
    primary key (card_id, image_type_id),
    foreign key (card_id) references card(id),
    foreign key (image_type_id) references image_type(id)
);

create table if not exists mtg_type (
    id integer primary key,
    name text not null unique
);

create table if not exists card_type_map (
    card_id text not null,
    type_id integer not null,
    primary key (card_id, type_id),
    foreign key (card_id) references card(id),
    foreign key (type_id) references mtg_type(id)
);

create table if not exists format (
    id integer primary key,
    name text not null unique
);

create table if not exists legality (
    card_id text not null,
    format_id integer not null,
    is_legal boolean not null,
    primary key (card_id, format_id),
    foreign key (card_id) references card(id),
    foreign key (format_id) references format(id)
);

create index if not exists idx_card on card(id);
