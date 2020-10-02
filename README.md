# ormx
Lightweight macros for [sqlx](https://github.com/launchbadge/sqlx)
## getting started
Add `ormx` and `sqlx` to your `Cargo.toml`
```toml
[dependencies.ormx]
git = "https://github.com/NyxCode/ormx"
features = ["mysql"]

[dependencies.sqlx]
version = "0.4.0-beta.1"
features = ["macros", "mysql"]
```
Right now, `ormx` only supports with mysql.
## what does it do? 
`ormx` provides macros for generating commonly used sql queries at compile time.  
`ormx` is meant to be used together with `sqlx`. Everything it generates uses `sqlx::query!` under the hood, so every generated query will be checked against your database at compile time.  
## what doesn't it do?
`ormx` is not a full-fledged ORM nor a query builder. For everything except simple CRUD, you can always just use `sqlx`.  
## [example](https://github.com/NyxCode/ormx/tree/master/example/src/main.rs)
