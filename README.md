# sqlx-crud

sqlx-crud is an extension to [SQLx](https://github.com/launchbadge/sqlx) to
derive Create, Read, Update, and Delete (CRUD) methods for a struct
representing a table in a sqlx database.

```rust
use sqlx::FromRow;
use sqlx_crud::Crud;

#[derive(Debug, FromRow, Crud)]
struct User {
    user_id: i32,
    name: String,
}

if let Some(user) = User::by_id(&pool, 42) {
    println!("Found user user_id=42: {:?}", user);
}
```

### Notable Features

* **Single Derive Macro for Structs**

* **Methods to Create, Read, Update, and Delete Records**

* **Primary Key and Table Name Inference**

* **Table Metadata for Reuse**

This removes much of the common, repetitive code needed when dealing with
the typical CRUD operations.

sqlx-crud strives to do a few, narrowly defined things well in an effort
to reduce 80% of the redundant code you might write for a database
application when operating on a single table at a time. It provides
mechanisms: [Schema](./src/traits.rs) and [Crud](./src/traits.rs), to access
and reuse the generated id, column, and query metadata. This can help with
writing more complex queries outside of the single table CRUD paradigm, but
its primary use case is for CRUD.

## Installation

Installing sqlx-crud is similar to installing SQLx.

```toml
# Cargo.toml
[dependencies]
sqlx-crud = { version = "0", features = ["runtime-tokio-rustls"] }
```

See the [documentation](https://docs.rs/sqlx-crud/latest) for full usage
instructions.

## Features

The features are the same as SQLx `runtime-*` flags and are required because of
the dependency on SQLx. Hopefully this constraint will be removed in the future.

## Source code

Source code for sqlx-crud is available at [https://www.github.com/treydempsey/sqlx-crud](https://www.github.com/treydempsey/sqlx-crud).

## Documentation

Documentation is hosted at [sqlx-crud docs](https://docs.rs/sqlx-crud/latest).

## Roadmap

sqlx-crud does most of what I need it to do, however while packaging it for
release to others I realized there are several improvements that could be made.
Planned updates and major achievements are listed in [MILESTONES](./MILESTONES).

## License

sqlx-crud is licensed under the MIT license (see: [LICENSE](./LICENSE)).
