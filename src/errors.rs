use thiserror::Error;

#[derive(Error, Debug)]
pub enum AvisError {
    #[error("Unsupported mesh topology")]
    UnsupportedMeshTopology,
    #[error("Database connection error: {:?}", 0)]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Database connection pool error: {:?}", 0)]
    DatabasePoolError(#[from] r2d2::Error),
}

pub type Result<X> = std::result::Result<X, AvisError>;
