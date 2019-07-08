-- This file should undo anything in `up.sql`
ALTER TABLE products RENAME COLUMN cost TO price;

DROP TABLE prices_products;
DROP TABLE prices;