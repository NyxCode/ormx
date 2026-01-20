-- Test schema-qualified table names
CREATE SCHEMA IF NOT EXISTS app;

CREATE TABLE app.products
(
    id    SERIAL PRIMARY KEY,
    name  VARCHAR(128) NOT NULL,
    price FLOAT8 NOT NULL
);
