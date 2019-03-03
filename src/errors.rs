use failure::{Error,Fail};

#[derive(Debug, Fail)]
pub enum AmbleError {
    #[fail(display = "IoError: {}", _0)]
    IoError (String),

    #[fail(display = "SystemTimeError: {}", _0)]
    SystemTimeError (String),

    #[fail(display = "WalkDir: {}", _0)]
    WalkDirError (String),
}

impl From<std::io::Error> for AmbleError {
    fn from(error: std::io::Error) -> Self {
        AmbleError::IoError(error.to_string())
    }
}

impl From<std::time::SystemTimeError> for AmbleError {
    fn from(error: std::time::SystemTimeError) -> Self {
        AmbleError::SystemTimeError(error.to_string())
    }
}

impl From<walkdir::Error> for AmbleError {
    fn from(error: walkdir::Error) -> Self {
        AmbleError::WalkDirError(error.to_string())
    }
}

