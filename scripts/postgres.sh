CONTAINER_ID=$(
  docker run -it --rm --name ormx-test-postgres-db \
    -e POSTGRES_DB=ormx \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=admin \
    -v $(pwd)/scripts/postgres-schema.sql:/docker-entrypoint-initdb.d/schema.sql \
    -d postgres
)
CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $CONTAINER_ID)
echo "DATABASE_URL=postgres://postgres:admin@$CONTAINER_IP/ormx" > .env
