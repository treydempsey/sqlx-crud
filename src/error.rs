use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Sqlx error {0}")]
    SqlxError(#[from] sqlx::Error),
}
