echo "DATABASE_URL=mysql://root:admin@127.0.0.1/ormx" > .env

docker run -it --rm --name ormx-test-mysql-db \
  -e MYSQL_DATABASE=ormx \
  -e MYSQL_ROOT_PASSWORD=admin \
  -p 3306:3306 \
  mysql:latest