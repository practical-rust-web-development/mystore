-- Your SQL goes here
CREATE TABLE products (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  stock TEXT NOT NULL,
  price INTEGER --representing cents
)