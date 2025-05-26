create table if not exists card (
    id text not null primary key,
    object text not null,
    name text not null,
    color_indicator text,
    produced_mana text,
    loyalty integer,
    artist text,
    oracle_id text,
    type_line text,
    lang text,
    content_warning boolean,
    converted_mana_cost real,
    image_status text,
    flavor_text text,
    arena_id integer,
    illustration_id text,
    oracle_text text,
    colors integer,
    color_identity integer,
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

create table if not exists card_supertype (
    card_id text not null references card(id),
    supertype text not null,
    primary key (card_id, supertype)
);

create table if not exists card_type (
    card_id text not null references card(id),
    type text not null,
    primary key (card_id, type)
);

create table if not exists card_subtype (
    card_id text not null references card(id),
    subtype text not null,
    primary key (card_id, subtype)
);

create table if not exists format (
    id integer primary key,
    name text not null unique
);

create table if not exists legality (
    card_id text not null,
    format_id integer not null,
    status text not null check (status in ('legal', 'banned', 'restricted', 'not_legal')),
    primary key (card_id, format_id),
    foreign key (card_id) references card(id),
    foreign key (format_id) references format(id)
);

create table if not exists keyword (
    id integer primary key,
    name text not null unique
);

create table if not exists card_keywords (
    card_id text not null,
    keyword_id integer not null,
    primary key (card_id, keyword_id),
    foreign key (card_id) references card(id),
    foreign key (keyword_id) references keyword(id)
);

create index if not exists idx_card_name on card(name);
create index if not exists idx_card_set_id on card(set_id);
create index if not exists idx_card_rarity on card(rarity);
create index if not exists idx_card_oracle_id on card(oracle_id);
create index if not exists idx_card_colors on card(colors);
create index if not exists idx_card_color_identity on card(color_identity);
create index if not exists idx_card_converted_mana_cost on card(converted_mana_cost);
create index if not exists idx_card_power on card(power);
create index if not exists idx_card_toughness on card(toughness);
create index if not exists idx_card_arena on card(arena);
create index if not exists idx_card_mtgo on card(mtgo);
create index if not exists idx_card_paper on card(paper);
create index if not exists idx_card_supertype_type on card_supertype(supertype);
create index if not exists idx_card_type_type on card_type(type);
create index if not exists idx_card_subtype_type on card_subtype(subtype);
create index if not exists idx_legality_format_status on legality(format_id, status);
create index if not exists idx_legality_status on legality(status);
create index if not exists idx_card_keywords_keyword on card_keywords(keyword_id);
create index if not exists idx_card_cmc_types on card(converted_mana_cost, set_id);
create index if not exists idx_card_set_rarity on card(set_id, rarity);
