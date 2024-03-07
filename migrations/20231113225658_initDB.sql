-- Add migration script here
CREATE TABLE tbl_user (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);

CREATE TABLE tbl_accounting (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    name TEXT NOT NULL,
    user_id UUID NOT NULL,
    FOREIGN KEY(user_id) REFERENCES tbl_user(id)
);

CREATE TABLE tbl_type (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    parent_type UUID,
    name TEXT NOT NULL,
    user_id uuid,
    FOREIGN KEY(parent_type) REFERENCES tbl_type(id),
    FOREIGN KEY(user_id) REFERENCES tbl_user(id)
);

CREATE TYPE MOVEMENT_DIRECTION AS ENUM ('in', 'out');

CREATE TABLE accounting_movement_tbl (
    accounting_id UUID NOT NULL,
    type_id UUID NOT NULL,
    direction MOVEMENT_DIRECTION NOT NULL,
    amount NUMERIC(14,2) NOT NULL,
    FOREIGN KEY(accounting_id) REFERENCES tbl_accounting(id),
    FOREIGN KEY(type_id) REFERENCES tbl_type(id)
);
