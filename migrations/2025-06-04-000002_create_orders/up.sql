CREATE TABLE orders (
    id CHAR(36) PRIMARY KEY,
    customer_id CHAR(36) NOT NULL,
    total_amount DOUBLE NOT NULL,
    created_at TIMESTAMP NOT NULL
);