use std::task::{Waker, Context};
use crossbeam_utils::sync::{Parker, Unparker};
use crate::error::Result;

pub enum Notifier {
    Unparker(Unparker),
    Waker(Waker)
}

impl Notifier {
    pub fn notify(self) {
        match self {
            Self::Unparker(unparker) => unparker.unpark(),
            Self::Waker(waker) => waker.wake()
        }
    }
}

pub struct Inner<T> {
    pub data: Option<Result<T>>,
    pub notifier: Option<Notifier>
}

impl<T> Inner<T> {
    pub fn new() -> *mut Self {
        Box::into_raw(Box::new(Self {
            data: None,
            notifier: None
        })) as *mut _
    }
}

pub struct Waiter<T> {
    inner: *mut Inner<T>
}

impl<T> Waiter<T> {
    pub fn new(inner: *mut Inner<T>) -> Self {
        Self {
            inner
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.inner.is_null()
    }

    pub fn inner(&mut self) -> Option<&mut Inner<T>> {
        if self.is_valid() {
            Some(unsafe { &mut *self.inner })
        } else {
            None
        }
    }

    pub fn try_get(&mut self) -> Option<Result<T>> {
        if !self.is_valid() {
            panic!("Result dropped");
        } else {
            self.inner().map(|inner| inner.data.take())?
        }
    }

    pub fn wait(&mut self) -> Result<T> {
        if let Some(item) = self.try_get() {
            return item;
        }

        let parker = Parker::new();
        self.inner().map(|inner| {
            inner.notifier = Some(Notifier::Unparker(parker.unparker().clone()));
        });
        parker.park();

        self.try_get().unwrap()
    }

    pub fn set_waker(&mut self, cx: &Context) {
        self.inner().map(|inner| {
            inner.notifier = Some(Notifier::Waker(cx.waker().clone()))
        });
    }
}

impl<T> Drop for Waiter<T> {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { let _ = Box::from_raw(self.inner); }
        }
    }
}
