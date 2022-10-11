use std::any::Any;

/// The error that can be returned after spawning a task.
/// This will be only seen when the provided task panics or the pool is stopped before the task
/// could be executed.
#[derive(Debug)]
pub enum Error {
    /// The task has panicked and the error is returned
    Panicked(Box<dyn Any + Send + 'static>),
    /// The task has been aborted, this is seen when the pool was stopped and the task didn't
    /// get to be executed before stopping.
    Aborted
}

impl From<Box<dyn Any + Send + 'static>> for Error {
    fn from(err: Box<dyn Any + Send + 'static>) -> Self {
        Error::Panicked(err)
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
