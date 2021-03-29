CREATE TABLE users
(
    id         INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
    first_name VARCHAR(128) NOT NULL,
    last_name  VARCHAR(128) NOT NULL,
    email      VARCHAR(128) NOT NULL UNIQUE,
    disabled   VARCHAR(128),
    last_login DATETIME DEFAULT NULL
)