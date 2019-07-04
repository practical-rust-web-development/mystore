-- Your SQL goes here
ALTER TABLE products ADD COLUMN text_searchable_product_col tsvector NOT NULL;

UPDATE products SET text_searchable_product_col = 
    to_tsvector('english', name || ' ' || coalesce(description, ''));

CREATE INDEX textsearch_idx ON products USING RUM (text_searchable_product_col rum_tsvector_ops);

CREATE TRIGGER tsvectorupdateproducts BEFORE INSERT OR UPDATE
ON products FOR EACH ROW EXECUTE PROCEDURE
tsvector_update_trigger(text_searchable_product_col, 'pg_catalog.english', name, description);