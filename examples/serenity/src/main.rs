use serenity::async_trait;
use serenity::prelude::*;
use sqlx::{FromRow, Pool, Sqlite, SqlitePool};
use sqlx_crud::SqlxCrud;

struct Handler {
    db: Pool<Sqlite>,
}

#[derive(FromRow, SqlxCrud)]
struct User {
    id: i64,
    name: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, _msg: serenity::model::channel::Message) {
        use sqlx_crud::Crud;

        // Taken from https://github.com/treydempsey/sqlx-crud/issues/9#issuecomment-1509718232

        let r = User {
            id: 1,
            name: "test".to_owned(),
        }
        .create(&self.db)
        .await;

        assert!(r.is_ok());
        let user = r.unwrap();
        assert_eq!(1, user.id);
        assert_eq!("test", user.name);

        let r = User::by_id(&self.db, 1).await;
        assert!(r.is_ok());
        let o = r.unwrap();
        assert!(o.is_some());
        let user = o.unwrap();
        assert_eq!(1, user.id);
        assert_eq!("test", user.name);

        let mut user = user;
        user.name = "vogon".to_string();
        let r = user.update(&self.db).await;
        assert!(r.is_ok());
        let user = r.unwrap();
        assert_eq!(1, user.id);
        assert_eq!("vogon", user.name);

        let r = User::by_id(&self.db, 1).await;
        assert!(r.is_ok());
        let o = r.unwrap();
        assert!(o.is_some());
        let user = o.unwrap();
        assert_eq!(1, user.id);
        assert_eq!("vogon", user.name);

        let r = user.delete(&self.db).await;
        assert!(r.is_ok());

        let r = User::by_id(&self.db, 1).await;
        assert!(r.is_ok());
        let o = r.unwrap();
        assert!(o.is_none());
    }
}

#[tokio::main]
async fn main() {
    let db = SqlitePool::connect("sqlite:sqlite.db").await.unwrap();

    let _ = Handler { db };
}
