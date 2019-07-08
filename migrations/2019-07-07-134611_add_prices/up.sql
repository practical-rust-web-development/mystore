-- Your SQL goes here
CREATE TABLE prices (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users(id),
  CHECK (name <> '')
);

CREATE TABLE prices_products (
  id SERIAL PRIMARY KEY,
  price_id INTEGER NOT NULL REFERENCES prices(id),
  product_id INTEGER NOT NULL REFERENCES products(id),
  user_id INTEGER NOT NULL REFERENCES users(id),
  amount INTEGER --representing cents
);

ALTER TABLE products RENAME COLUMN price TO cost;