use crate::wait::Waiter;
use crate::error::Result;
use std::{future::Future, pin::Pin, task::{Context, Poll}};
use std::marker::PhantomData;
use std::sync::Arc;
use parking_lot::Mutex;


/// A handle used to retrieve the output of a task.
///
/// The output can be waited synchronously by using [`wait`] or the handle can be `.await`ed
/// to wait for the result asynchronously.
///
/// [`wait`]: JoinHandle::wait
#[must_use = "If you don't want the result, use spawn_detached."]
pub struct JoinHandle<T> {
    pub(crate) inner: Waiter<T>
}

unsafe impl<T> Send for JoinHandle<T> {}

impl<T> JoinHandle<T> {
    /// Waits for the result synchronously.
    pub fn wait(mut self) -> Result<T> {
        self.inner.wait()
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(item) = self.inner.try_get() {
            Poll::Ready(item)
        } else {
            self.inner.set_waker(cx);
            Poll::Pending
        }
    }
}

pub struct ScopedJoinHandle<'scope, T> {
    pub(crate) join: JoinHandle<()>,
    pub(crate) mutex: Arc<Mutex<Option<T>>>,
    pub(crate) _marker: PhantomData<&'scope T>
}

impl<T> ScopedJoinHandle<'_, T> {
    pub fn join(self) -> Result<T> {
        self.join.wait()?;
        Ok(match Arc::try_unwrap(self.mutex) {
            // The output has already been put inside the option by the task, so it can't panic
            // when unwrapping.
            Ok(mutex) => mutex.into_inner().unwrap(),
            // By now, the other task holding the Arc has exited and we are the only ones holding
            // it, so unwrapping it cannot fail.
            Err(_) => unreachable!()
        })
    }
}
