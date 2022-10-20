use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use crate::core::Core;
use crate::{JoinHandle, Runnable};
use crate::periodic::PeriodicTask;
use crate::scope::Scope;
use crate::sync::Task;
use crate::wait::{Inner, Waiter};

/// Handle used to operate the pool.
#[derive(Clone)]
pub struct Handle {
    pub(crate) core: Arc<Core>
}

impl Handle {
    /// Tries to get the current handle, panicking if not inside the pool context.
    pub fn current() -> Self {
        Self::try_current().expect("Not inside of a worker pool context")
    }

    /// Tries to get the current handle, returning [`None`] if not inside the pool context.
    ///
    /// [`None`]: std::option::Option::None
    pub fn try_current() -> Option<Self> {
        crate::context::try_get()
    }

    /// Spawns a new task into the pool, returning a [`handle`] that can be used to retrieve the output.
    ///
    /// [`handle`]: crate::join::JoinHandle
    pub fn spawn<R>(&self, runnable: R) -> JoinHandle<R::Output>
    where
        R: Runnable
    {
        let inner = Inner::<R::Output>::new();
        let task = Task::new(runnable, Some(inner));
        self.core.schedule(task);
        JoinHandle {
            inner: Waiter::new(inner)
        }
    }

    /// Like [`spawn`], spawns a new task into the pool, but doesn't return a handle, so the output
    /// cannot be retrieved and the allocation needed to do so is skipped.
    ///
    /// [`spawn`]: crate::spawn
    pub fn spawn_detached<R>(&self, runnable: R)
    where
        R: Runnable
    {
        let task = Task::new(runnable, None);
        self.core.schedule(task);
    }

    /// Spawns a new task that will be executed periodically by the thread pool every specified time
    /// and the specified amount of times.
    pub fn spawn_periodic<T>(&self, task: T, every: Duration, times: Option<usize>)
    where
        T: Fn() + Send + 'static
    {
        let task = PeriodicTask::new(self.clone(), task, every, times);
        self.core.schedule_periodical(task);
    }

    pub fn scoped<'scope, 'env: 'scope, F, R>(&'scope self, fun: F) -> R
    where
        F: for<'a> FnOnce(&'a Scope<'scope, 'env>) -> R
    {
        let scope = Scope::new(self);
        let result = fun(&scope);

        scope.wait();

        result
    }

    /// Shuts down the pool, waiting for all threads to exit.
    pub fn shutdown(self) {
        self.core.shutdown();
    }

    /// Enters the context of the pool the handle belongs to, thus allowing to use directly
    /// [`spawn`]/[`spawn_detached`]/[`spawn_periodic`] without using the handle.
    ///
    /// [`spawn`]: crate::spawn
    /// [`spawn_detached`]: crate::spawn_detached
    /// [`spawn_periodic`]: crate::spawn_periodic
    pub fn enter_context(&self) -> ContextGuard {
        if crate::context::try_get().is_some() {
            panic!("Already inside the context of a worker pool");
        }

        crate::context::set(self.clone());
        ContextGuard(PhantomData)
    }
}

pub struct ContextGuard<'a>(PhantomData<&'a Handle>);

impl Drop for ContextGuard<'_> {
    fn drop(&mut self) {
        crate::context::clear();
    }
}
