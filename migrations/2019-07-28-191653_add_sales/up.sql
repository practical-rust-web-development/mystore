-- Your SQL goes here
CREATE TABLE sales (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  sale_date DATE NOT NULL,
  total FLOAT NOT NULL
);

CREATE TABLE sale_products (
  id SERIAL PRIMARY KEY,
  product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  sale_id INTEGER NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
  amount FLOAT NOT NULL,
  discount INTEGER NOT NULL,
  tax INTEGER NOT NULL,
  price INTEGER NOT NULL, --representing cents
  total FLOAT NOT NULL
)
