use std::sync::Arc;
use std::time::Duration;
use crate::core::Core;
use crate::{JoinHandle, Runnable};
use crate::periodic::PeriodicTask;
use crate::sync::Task;
use crate::wait::{Inner, Waiter};

#[derive(Clone)]
pub struct Handle {
    pub(crate) core: Arc<Core>
}

impl Handle {
    pub fn current() -> Self {
        Self::try_current().unwrap()
    }

    pub fn try_current() -> Option<Self> {
        crate::context::try_get()
    }

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

    pub fn spawn_detached<R>(&self, runnable: R)
    where
        R: Runnable
    {
        let task = Task::new(runnable, None);
        self.core.schedule(task);
    }

    pub fn spawn_periodic<T>(&self, task: T, every: Duration, times: Option<usize>)
    where
        T: Fn() + Send + 'static
    {
        let task = PeriodicTask::new(self.clone(), task, every, times);
        self.core.schedule_periodical(task);
    }

    pub fn shutdown(self) {
        self.core.shutdown();
    }

    pub fn enter_context(&self) {
        if crate::context::try_get().is_some() {
            panic!("Already inside the context of a worker pool");
        }

        crate::context::set(self.clone());
    }
}
