use tiny_fn::tiny_fn;
use crate::wait::{Inner, Shared};
use crate::error::Result;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::NonNull;
use crate::Runnable;

tiny_fn! {
    struct TaskFun = FnOnce();
}

pub struct Task {
    fun: Option<TaskFun<'static>>,
    inner: Option<Shared<()>>,
    abort: fn(NonNull<Inner<()>>),
    can_run: fn(NonNull<Inner<()>>) -> bool
}

impl Task {
    pub fn new<R>(fun: R, ptr: Option<Shared<R::Output>>) -> Self
    where
        R: Runnable
    {
        let inner = ptr.as_ref().map(|i| i.as_ptr());
        Self {
            fun: Some(TaskFun::new(move || {
                let res = catch_unwind(AssertUnwindSafe(|| fun.run()))
                    .map_err(Into::into);
                inner.map(move |ptr| unsafe {
                    set_result(ptr, res);
                });
            })),
            inner: ptr.map(|inner| inner.cast()),
            abort: abort_task::<R::Output>,
            can_run: Self::_can_run::<R::Output>
        }
    }

    pub fn abort(mut self) {
        self.inner.take().map(|ptr| (self.abort)(ptr.as_ptr()));
    }

    pub fn can_run(&self) -> bool {
        self.inner.as_ref().map(|i| {
            (self.can_run)(i.as_ptr())
        }).unwrap_or(true)
    }

    fn _can_run<T>(ptr: NonNull<Inner<()>>) -> bool {
        unsafe {
            !ptr.cast::<Inner<T>>().as_ref().state.is_aborted()
        }
    }

    pub fn run(mut self) {
        if !self.can_run() {
            return;
        }
        self.fun.take().map(|f| f.call());
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        if self.inner.as_ref().map(|i| unsafe { i.drop_end() }).unwrap_or(false) {
            unsafe { let _ = Box::from_raw(self.inner.take().unwrap().as_ptr().as_ptr()); }
        }
    }
}

unsafe fn set_result<T>(ptr: NonNull<Inner<T>>, result: Result<T>) {
    let inner = &mut *ptr.as_ptr();
    inner.finish(result);
    inner.notifier.take().map(|notifier| notifier.notify());
}

pub fn abort_task<T>(ptr: NonNull<Inner<()>>) {
    unsafe {
        let inner = &mut *ptr.cast::<Inner<T>>().as_ptr();
        inner.state.abort();
        inner.notifier.take().map(|n| n.notify());
    }
}
