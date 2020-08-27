CONTAINER_ID=$(
  docker run -it --rm --name ormx-test-mysql-db \
    -e MYSQL_DATABASE=ormx \
    -e MYSQL_ROOT_PASSWORD=admin \
    -v $(pwd)/schema.sql:/docker-entrypoint-initdb.d/schema.sql \
    -d mariadb
)
CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $CONTAINER_ID)
echo "DATABASE_URL=mysql://root:admin@$CONTAINER_IP/ormx" > .env
docker attach $CONTAINER_ID