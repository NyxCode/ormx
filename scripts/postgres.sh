echo "DATABASE_URL=postgres://postgres:admin@127.0.0.1/ormx" > .env

docker run -it --rm --name ormx-test-postgres-db \
  -e POSTGRES_DB=ormx \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=admin \
  -p 5432:5432 \
  postgres:latest \
  -c log_statement=all