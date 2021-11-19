## Postgres ORMX Example

This is an example that shows ormx querying a sqlite database.  The
ormx queries are shown in 'src/main.rs`.

To try this example, run:
```bash
# changet to the example-postgres directory
cd ormx/example-postgres
# start a postgres container, create tables, and writes `DATABASE_URL` in `.env` file.
../scripts/postgres.sh 
# run the example
cargo run
```
By default, the database file is `/tmp/example.db`.  To change it, edit the `.env` file in this directory.
