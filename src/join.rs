use crate::wait::Waiter;
use crate::error::Result;
use std::{future::Future, pin::Pin, task::{Context, Poll}};
use std::marker::PhantomData;
use std::sync::Arc;
use crossbeam_queue::SegQueue;
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

    pub fn abort(&self) {
        self.inner.abort();
    }

    pub fn is_aborted(&self) -> bool {
        self.inner.inner.state.is_aborted()
    }

    pub fn is_running(&self) -> bool {
        self.inner.inner.state.is_running()
    }

    pub fn is_finished(&self) -> bool {
        self.inner.inner.state.is_finished()
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

pub struct ScopedJoinHandle<'scope, 'env, T> {
    pub(crate) join: Option<JoinHandle<()>>,
    pub(crate) mutex: Option<Arc<Mutex<Option<T>>>>,
    pub(crate) queue: &'scope SegQueue<JoinHandle<()>>,
    pub(crate) _marker: PhantomData<&'scope &'env ()>
}

impl<T> ScopedJoinHandle<'_, '_, T> {
    pub fn join(mut self) -> Result<T> {
        self.join.take().unwrap().wait()?;
        Ok(match Arc::try_unwrap(self.mutex.take().unwrap()) {
            // The output has already been put inside the option by the task, so it can't panic
            // when unwrapping.
            Ok(mutex) => mutex.into_inner().unwrap(),
            // By now, the other task holding the Arc has exited and we are the only ones holding
            // it, so unwrapping it cannot fail.
            Err(_) => unreachable!()
        })
    }
}

impl<T> Drop for ScopedJoinHandle<'_, '_, T> {
    fn drop(&mut self) {
        if let Some(inner) = self.join.take() {
            self.queue.push(inner);
        }
    }
}

pub struct PeriodicJoinHandle {
    pub(crate) inner: Waiter<()>
}

impl PeriodicJoinHandle {
    pub fn abort(&self) {
        self.inner.abort()
    }

    pub fn is_aborted(&self) -> bool {
        self.inner.inner.state.is_aborted()
    }

    pub fn is_running(&self) -> bool {
        self.inner.inner.state.is_running()
    }

    pub fn is_finished(&self) -> bool {
        self.inner.inner.state.is_finished()
    }
}
