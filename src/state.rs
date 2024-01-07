use crate::error::{Error, Result};

#[derive(Default)]
pub enum State<T> {
    #[default]
    Started,
    Aborted,
    Finished(Option<Result<T>>)
}

impl<T> State<T> {
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Finished(_))
    }

    pub fn is_running(&self) -> bool {
        !self.is_finished() && !self.is_aborted()
    }

    pub fn is_aborted(&self) -> bool {
        matches!(self, Self::Aborted)
    }

    pub fn abort(&mut self) {
        *self = Self::Aborted;
    }

    pub fn get_output(&mut self) -> Option<Result<T>> {
        match self {
            Self::Finished(o) => o.take(),
            Self::Aborted => Some(Err(Error::Aborted)),
            _ => None
        }
    }
}
