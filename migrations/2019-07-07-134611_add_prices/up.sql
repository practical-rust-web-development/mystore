-- Your SQL goes here
CREATE TABLE prices (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  CHECK (name <> '')
);

CREATE TABLE prices_products (
  id SERIAL PRIMARY KEY,
  price_id INTEGER NOT NULL REFERENCES prices(id),
  product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  amount INTEGER, --representing cents
  UNIQUE (price_id, product_id)
);

ALTER TABLE products RENAME COLUMN price TO cost;