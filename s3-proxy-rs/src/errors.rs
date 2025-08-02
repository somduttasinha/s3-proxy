use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] std::io::Error),
    #[error("Internal error: {0}")]
    InternalError(String),
}
