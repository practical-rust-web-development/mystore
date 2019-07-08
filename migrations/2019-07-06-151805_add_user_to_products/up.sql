-- Your SQL goes here
ALTER TABLE products ADD COLUMN user_id INTEGER NOT NULL;
ALTER TABLE products ADD CONSTRAINT products_user_id_foreign_key FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;