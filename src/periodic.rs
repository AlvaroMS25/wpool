use std::ptr::NonNull;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tiny_fn::tiny_fn;
use crate::handle::Handle;
use crate::wait::Shared;

tiny_fn! {
    struct PeriodicFn = Fn();
}

pub struct PeriodicTask {
    handle: Handle,
    fun: PeriodicFn<'static>,
    every: Duration,
    next: Instant,
    times: Option<usize>,
    inner: Option<Shared<()>>
}

impl PeriodicTask {
    pub fn new<F>(handle: Handle, fun: F, every: Duration, times: Option<usize>, inner: Option<Shared<()>>) -> Self
    where
        F: Fn() + Send + 'static
    {
        let next = Instant::now() + every;
        Self {
            handle,
            fun: PeriodicFn::new(fun),
            every,
            next,
            times,
            inner
        }
    }

    pub fn run(mut self) {
        if self.inner.as_ref().map(|i| i.state.is_aborted()).unwrap_or(false) {
            return;
        }

        self.fun.call();
        self.times.as_mut().map(|t| *t = *t-1);

        if self.times.is_none() || self.times.as_ref().map(|t| *t >= 1).unwrap() {
            self.next = Instant::now() + self.every;
            self.reschedule();
        }
    }

    pub fn reschedule(self) {
        if self.inner.as_ref().map(|i| unsafe { i.state.is_aborted() }).unwrap_or(false) {
            return; // if aborted, dont reschedule
        }
        // SAFETY: We already hold an Arc, so the pointer must be valid and safe to dereference.
        unsafe { (&*Arc::as_ptr(&self.handle.core)).schedule_periodical(self); }
    }

    pub fn can_run(&self) -> bool {
        Instant::now() >= self.next
    }
}

impl Drop for PeriodicTask {
    fn drop(&mut self) {
        if self.inner.as_ref().map(|i| unsafe { i.drop_end() }).unwrap_or(false) {
            unsafe { let _ = Box::from_raw(self.inner.take().unwrap().0.as_ptr()); }
        }
    }
}
