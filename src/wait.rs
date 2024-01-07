use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU8, Ordering};
use std::task::{Waker, Context};
use crossbeam_utils::sync::{Parker, Unparker};
use crate::error::Result;
use crate::state::State;

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

pub struct Shared<T>(pub NonNull<Inner<T>>);

impl<T> Shared<T> {
    pub fn new() -> Self {
        Self(NonNull::new(Inner::new()).unwrap())
    }

    #[allow(unused)]
    pub fn inner_mut(&self) -> &mut <Self as Deref>::Target {
        unsafe { &mut *self.0.as_ptr() }
    }

    pub fn as_ptr(&self) -> NonNull<Inner<T>> {
        self.0
    }

    pub fn cast<N>(self) -> Shared<N> {
        Shared(self.0.cast::<Inner<N>>())
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        self.create_end();
        Self(self.0)
    }
}

impl<T> Deref for Shared<T> {
    type Target = Inner<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

pub struct Inner<T> {
    pub notifier: Option<Notifier>,
    pub state: State<T>,
    pub handles: AtomicU8
}

impl<T> Inner<T> {
    pub fn new() -> *mut Self {
        Box::into_raw(Box::new(Self {
            notifier: None,
            state: State::default(),
            handles: AtomicU8::new(1)
        })) as *mut _
    }

    pub fn finish(&mut self, data: Result<T>) {
        self.state = State::Finished(Some(data));
    }

    pub fn create_end(&self) {
        self.handles.fetch_add(1, Ordering::Release);
    }

    pub fn drop_end(&self) -> bool {
        self.handles.fetch_sub(1, Ordering::Acquire) <= 1
    }
}

#[cfg(test)]
impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        println!("Dropping inner");
    }
}

pub struct Waiter<T> {
    pub(crate) inner: Shared<T>
}

impl<T> Waiter<T> {
    pub fn new(inner: Shared<T>) -> Self {
        Self {
            inner
        }
    }

    pub fn inner(&mut self) -> &mut Shared<T> {
        &mut self.inner
    }

    pub fn try_get(&mut self) -> Option<Result<T>> {
        let inner = self.inner();
        inner.state.get_output()
    }

    pub fn wait(&mut self) -> Result<T> {
        if let Some(item) = self.try_get() {
            return item;
        }

        let parker = Parker::new();
        let inner = self.inner();
        inner.notifier = Some(Notifier::Unparker(parker.unparker().clone()));
        parker.park();

        self.try_get().unwrap()
    }

    pub fn set_waker(&mut self, cx: &Context) {
        let inner = self.inner();
        inner.notifier = Some(Notifier::Waker(cx.waker().clone()))
    }

    pub fn abort(&self) {
        crate::sync::abort_task::<T>(self.inner.as_ptr().cast());
    }
}

impl<T> Drop for Waiter<T> {
    fn drop(&mut self) {
        if self.inner.drop_end() {
            unsafe { let _ = Box::from_raw(self.inner.as_ptr().as_ptr()); }
        }
    }
}
