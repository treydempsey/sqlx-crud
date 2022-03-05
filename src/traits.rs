use std::pin::Pin;

use futures::stream::Stream;
use futures::stream::TryCollect;
use futures::Future;
use futures::{future, TryFutureExt, TryStreamExt};
use sqlx::database::HasArguments;
use sqlx::query::Query;
use sqlx::{Database, Encode, Executor, FromRow, IntoArguments, Type};

pub type CrudFut<'e, T> = Pin<Box<dyn Future<Output = Result<T, sqlx::Error>> + 'e>>;
pub type CrudStream<'e, T> =
    Pin<Box<dyn Stream<Item = Result<T, sqlx::Error>> + std::marker::Send + 'e>>;
pub type TryCollectFut<'e, T> = TryCollect<CrudStream<'e, T>, Vec<T>>;

pub trait Schema {
    type Id: Send;

    fn table_name() -> &'static str;
    fn id(&self) -> Self::Id;
    fn id_column() -> &'static str;
    fn columns() -> &'static [&'static str];
    fn select_sql() -> &'static str;
    fn select_by_id_sql() -> &'static str;
    fn insert_sql() -> &'static str;
    fn update_by_id_sql() -> &'static str;
    fn delete_by_id_sql() -> &'static str;
}

pub trait Crud<'e, E>
where
    Self: 'e
        + Sized
        + Send
        + Unpin
        + for<'r> FromRow<'r, <E::Database as Database>::Row>
        + Schema,
    <Self as Schema>::Id:
        Encode<'e, <E as Executor<'e>>::Database> + Type<<E as Executor<'e>>::Database>,
    E: Executor<'e> + 'e,
    <E::Database as HasArguments<'e>>::Arguments: IntoArguments<'e, <E as Executor<'e>>::Database>,
{
    fn insert_binds(
        &'e self,
        query: Query<'e, E::Database, <E::Database as HasArguments<'e>>::Arguments>,
    ) -> Query<'e, E::Database, <E::Database as HasArguments<'e>>::Arguments>;

    fn update_binds(
        &'e self,
        query: Query<'e, E::Database, <E::Database as HasArguments<'e>>::Arguments>,
    ) -> Query<'e, E::Database, <E::Database as HasArguments<'e>>::Arguments>;

    fn create(&'e self, pool: E) -> CrudFut<'e, ()> {
        Box::pin(async move {
            let query = sqlx::query(<Self as Schema>::insert_sql());
            let query = self.insert_binds(query);
            query.execute(pool).await?;

            Ok(())
        })
    }

    fn all(pool: E) -> TryCollectFut<'e, Self> {
        let stream =
            sqlx::query_as::<E::Database, Self>(<Self as Schema>::select_sql()).fetch(pool);
        stream.try_collect()
    }

    fn paged(_pool: E) -> TryCollectFut<'e, Self> {
        unimplemented!()
    }

    fn by_id(pool: E, id: <Self as Schema>::Id) -> CrudFut<'e, Option<Self>> {
        Box::pin(
            sqlx::query_as::<E::Database, Self>(<Self as Schema>::select_by_id_sql())
                .bind(id)
                .fetch_optional(pool)
        )
    }

    fn update(&'e self, pool: E) -> CrudFut<'e, ()> {
        Box::pin(async move {
            let query = sqlx::query(<Self as Schema>::update_by_id_sql());
            let query = self.update_binds(query);
            query.execute(pool).await?;

            Ok(())
        })
    }

    fn delete(self, pool: E) -> CrudFut<'e, ()> {
        let query = sqlx::query(<Self as Schema>::delete_by_id_sql()).bind(self.id());
        Box::pin(
            query
                .execute(pool)
                .and_then(|_| future::ok(())),
        )
    }
}
