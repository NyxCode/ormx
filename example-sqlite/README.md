## Sqlite ORMX Example

This is an example that shows ormx querying a sqlite database.  The
ormx queries are shown in 'src/main.rs`.

To try this example, run:
```bash
cd ormx/example-sqlite
sqlx database setup
cargo run
```
By default, the database file is `/tmp/example.db`.  To change it, edit the `.env` file in this directory.
