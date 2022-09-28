use std::any::Any;

#[derive(Debug)]
pub enum Error {
    Panicked(Box<dyn Any + Send + 'static>),
    Aborted
}

impl From<Box<dyn Any + Send + 'static>> for Error {
    fn from(err: Box<dyn Any + Send + 'static>) -> Self {
        Error::Panicked(err)
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
