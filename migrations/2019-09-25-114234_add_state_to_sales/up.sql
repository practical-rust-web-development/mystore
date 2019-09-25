-- Your SQL goes here
CREATE TYPE sale_state AS ENUM ('draft', 'approved', 'not_payed', 'payed', 'cancelled');
ALTER TABLE sales ADD COLUMN state sale_state;
UPDATE sales SET state = 'approved'; 
ALTER TABLE sales ALTER COLUMN state SET NOT NULL;