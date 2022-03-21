//! sqlx-crud is an extension to [sqlx] to derive Create, Read, Update, and
//! Delete (CRUD) methods for a struct representing a table in a sqlx database.
//!
//! This removes much of the common, repetitive code needed when dealing with
//! the typical CRUD operations.
//!
//! This library strives to do a few, narrowly defined things well in an effort
//! to reduce 80% of the redundant code you might write for a database
//! application when operating on a single table at a time. It provides
//! mechanisms: [Schema] and [Crud], to access and reuse the generated id,
//! column, and query metadata. This can help with writing more complex queries
//! outside of the single table CRUD paradigm.
//!
//! [sqlx]: https://github.com/launchbadge/sqlx
//!
//! # Design Considerations
//!
//! The code currently assumes identifiers are assigned outside of the database.
//! This likely means the identifier is a UUID. Database generated IDs will be
//! added in a subsequent release.
//!
//! The primary key for the table can be indicated by use of the [sqlx_crud_macros::SqlxCrud]
//! `#[id]` field attribute. If no field is tagged as the [sqlx_crud_macros::SqlxCrud] `#[id]`
//! then the first field in the struct is assumed to be the ID.
//!
//! The ordering of the columns used by queries and which columns are present
//! is controlled by the field order of the struct. Ignored fields are not
//! currently supported but will be added.
//!
//! # Features
//!
//! Because sqlx-crud depends on sqlx you need to use the same executor and TLS
//! feature pair as you did with sqlx. If for example you used `tokio-rustls`
//! with sqlx you should also use the same feature with sqlx-crud.
//!
//! Hopefully I can figure out a way to remove this requirement. I think
//! I might need to use a build.rs script and interrogate the sqlx features that way.
//!
//! # Examples
//!
//! Given a table `users` defined as:
//!
//! ```sql
//! CREATE TABLE users (
//!     id INTEGER PRIMARY KEY NOT NULL,
//!     username TEXT NOT NULL
//! );
//! ```
//!
//! To define a `User` struct with generated [Crud] methods:
//!
//! ```rust
//! use sqlx::FromRow;
//! use sqlx_crud::SqlxCrud;
//!
//! #[derive(Debug, FromRow, SqlxCrud)]
//! pub struct User {
//!    pub user_id: i32,
//!    pub name: String,
//! }
//! ```
//!
//! [Crud]: traits/trait.Crud.html
//!
//! To create a new `User` in the database:
//!
//! ```rust
//! # use sqlx_crud::doctest::setup;
//! # use sqlx_crud::doctest::User;
//! use sqlx_crud::Crud;
//!
//! # tokio_test::block_on(async {
//! # let pool = setup().await;
//!
//! let new_user = User { user_id: 2, name: "new_user".to_string() };
//! new_user.create(&pool).await?;
//!
//! # Ok::<(), sqlx::Error>(())
//! # });
//! ```
//!
//! To query for a `User` where `user_id = 1`:
//!
//! ```rust
//! # use sqlx_crud::doctest::setup;
//! # use sqlx_crud::doctest::User;
//! use sqlx_crud::Crud;
//!
//! # tokio_test::block_on(async {
//! # let pool = setup().await;
//!
//! if let Some(user) = User::by_id(&pool, 1).await? {
//!     println!("User: {:?}", user);
//! }
//!
//! # Ok::<(), sqlx::Error>(())
//! # });
//! ```
//!
//!  To update an existing record:
//!
//!  ```rust
//! # use sqlx_crud::doctest::setup;
//! # use sqlx_crud::doctest::User;
//! use sqlx_crud::Crud;
//!
//! # tokio_test::block_on(async {
//! # let pool = setup().await;
//!
//! if let Some(mut user) = User::by_id(&pool, 1).await? {
//!     user.name = "something else".to_string();
//!     user.update(&pool).await?;
//! }
//!
//! # Ok::<(), sqlx::Error>(())
//! # });
//!  ```
//!
//! To delete a record:
//!
//! ```rust
//! # use sqlx_crud::doctest::setup;
//! # use sqlx_crud::doctest::User;
//! use sqlx_crud::Crud;
//!
//! # tokio_test::block_on(async {
//! # let pool = setup().await;
//!
//! if let Some(mut user) = User::by_id(&pool, 1).await? {
//!     user.delete(&pool).await?;
//! }
//!
//! # Ok::<(), sqlx::Error>(())
//! # });
//! ```
//!
//! Reusing and modifying the [select_sql] query string:
//!
//! ```rust
//! # use futures::stream::TryStreamExt;
//! # use sqlx::SqlitePool;
//! # use sqlx::FromRow;
//! use sqlx_crud::{Schema, SqlxCrud};
//!
//! #[derive(Debug, FromRow, SqlxCrud)]
//! pub struct User {
//!    pub user_id: i32,
//!    pub name: String,
//! }
//!
//! impl User {
//!     pub async fn all_limit(pool: &SqlitePool, limit: i32) -> Result<Vec<Self>, sqlx::Error> {
//!         let query = format!(
//!             "{} ORDER BY users.id ASC LIMIT ?",
//!             <Self as Schema>::select_sql()
//!         );
//!
//!         let mut users = Vec::new();
//!         let mut stream = sqlx::query_as::<_, Self>(&query)
//!             .bind(limit)
//!             .fetch(pool);
//!
//!         while let Some(user) = stream.try_next().await? {
//!             users.push(user);
//!         }
//!
//!         Ok(users)
//!     }
//! }
//! ```
//!
//! [select_sql]: traits/trait.Schema.html#tymethod.select_sql
//!
//! # Planned Future Improvements
//!
//! Subsequent updates will extend the library to be more useful in a larger
//! variety of situations.
//!
//! * Allow database assigned primary keys
//! * Crud::create() should return the assigned ID
//! * Add a field attribute to ignore fields

#[cfg(feature = "doctest")]
pub mod doctest;

pub mod error;
pub mod schema;
pub mod traits;

pub use error::Error;
pub use sqlx_crud_macros::SqlxCrud;
pub use traits::{Crud, Schema};
