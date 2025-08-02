use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("S3 connection error")]
    S3Error(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}
