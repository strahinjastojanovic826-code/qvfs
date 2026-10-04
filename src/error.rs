use std::fmt;

#[derive(Debug)]
pub enum QuatError {
    FileNotFound(String),
    PermissionDenied(String),
    InvalidHeader,
    TruncatedPayload,
    LockPoisoned,
    Io(std::io::Error),
}

impl fmt::Display for QuatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuatError::FileNotFound(path) => write!(f, "File not found: {path}"),
            QuatError::PermissionDenied(msg) => write!(f, "Permission denied: {msg}"),
            QuatError::InvalidHeader => write!(f, "Invalid header or magic bytes"),
            QuatError::TruncatedPayload => write!(f, "File truncated or missing quat payload"),
            QuatError::LockPoisoned => write!(f, "Internal concurrency error (lock poisoned)"),
            QuatError::Io(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl std::error::Error for QuatError {}

impl From<std::io::Error> for QuatError {
    fn from(err: std::io::Error) -> Self {
        QuatError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, QuatError>;