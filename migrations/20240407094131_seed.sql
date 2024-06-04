INSERT INTO tbl_type(id,parent_id,name,user_id)
VALUES
    -- car
    ('be1d078b-827b-438c-bd95-fbb5627115c3',NULL,'car',NULL),
        (DEFAULT,'be1d078b-827b-438c-bd95-fbb5627115c3','insurance',NULL),
        (DEFAULT,'be1d078b-827b-438c-bd95-fbb5627115c3','gas',NULL),
        (DEFAULT,'be1d078b-827b-438c-bd95-fbb5627115c3','mechanic',NULL),
    -- pet
    ('29606211-1175-4484-855e-404a25857eb7',NULL,'pet',NULL),
        (DEFAULT,'29606211-1175-4484-855e-404a25857eb7','food',NULL),
        (DEFAULT,'29606211-1175-4484-855e-404a25857eb7','health',NULL),
    -- home
    ('d7b94e25-2216-4bb2-ac9f-eeab9d2db342',NULL,'home',NULL),
    -- food
    ('b8c1b033-9e68-43b3-ac28-acffd1e7df5f',NULL,'food',NULL),
    -- extra
    ('cf1ccdc3-6b57-4a81-87ab-03328649745d',NULL,'extra',NULL),
        (DEFAULT,'cf1ccdc3-6b57-4a81-87ab-03328649745d','going out',NULL),
        (DEFAULT,'cf1ccdc3-6b57-4a81-87ab-03328649745d','shoes',NULL),
        (DEFAULT,'cf1ccdc3-6b57-4a81-87ab-03328649745d','vacations',NULL), -- add subtypes for different types of vacations
        (DEFAULT,'cf1ccdc3-6b57-4a81-87ab-03328649745d','cosmetics',NULL);
