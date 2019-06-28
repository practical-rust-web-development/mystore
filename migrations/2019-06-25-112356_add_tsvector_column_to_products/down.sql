-- This file should undo anything in `up.sql`
ALTER TABLE products DROP COLUMN text_searchable_product_col;

DROP TRIGGER tsvectorupdateproducts ON products;
