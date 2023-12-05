-- Add migration script here
CREATE TABLE tbl_user (
    id uuid NOT NULL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);

CREATE TABLE tbl_accounting (
    id SERIAL NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    user_id UUID NOT NULL,
    FOREIGN KEY(user_id) REFERENCES tbl_user(id)
);

CREATE TABLE tbl_type (
    id SERIAL NOT NULL PRIMARY KEY,
    parent_type INT,
    name TEXT NOT NULL,
    user_id uuid,
    FOREIGN KEY(parent_type) REFERENCES tbl_type(id),
    FOREIGN KEY(user_id) REFERENCES tbl_user(id)
);

CREATE TYPE movement_direction AS ENUM ('in', 'out');

CREATE TABLE accounting_movement_tbl (
    accounting_id INT NOT NULL,
    type_id int NOT NULL,
    direction movement_direction NOT NULL,
    amount NUMERIC(14,2) NOT NULL,
    FOREIGN KEY(accounting_id) REFERENCES tbl_accounting(id),
    FOREIGN KEY(type_id) REFERENCES tbl_type(id)
);
