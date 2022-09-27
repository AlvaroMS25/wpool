use std::sync::Arc;
use std::time::{Duration, Instant};
use tiny_fn::tiny_fn;
use crate::handle::Handle;

tiny_fn! {
    struct PeriodicFn = Fn();
}

pub struct PeriodicTask {
    handle: Handle,
    fun: PeriodicFn<'static>,
    every: Duration,
    next: Instant,
    times: Option<usize>
}

impl PeriodicTask {
    pub fn new<F>(handle: Handle, fun: F, every: Duration, times: Option<usize>) -> Self
    where
        F: Fn() + Send + 'static
    {
        let next = Instant::now() + every;
        Self {
            handle,
            fun: PeriodicFn::new(fun),
            every,
            next,
            times
        }
    }

    pub fn run(mut self) {
        self.fun.call();
        self.times.as_mut().map(|t| *t = *t-1);

        if self.times.is_none() || self.times.as_ref().map(|t| *t >= 1).unwrap() {
            self.next = Instant::now() + self.every;
            self.reschedule();
        }
    }

    pub fn reschedule(self) {
        unsafe { (&*Arc::as_ptr(&self.handle.core)).schedule_periodical(self); }
    }

    pub fn can_run(&self) -> bool {
        Instant::now() >= self.next
    }
}
