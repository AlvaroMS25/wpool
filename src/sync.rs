use tiny_fn::tiny_fn;
use crate::wait::Inner;
use crate::error::{Error, Result};
use std::panic::{catch_unwind, AssertUnwindSafe};
use crate::Runnable;

tiny_fn! {
    struct TaskFun = FnOnce();
}

pub struct Task {
    fun: TaskFun<'static>,
    inner: Option<*mut Inner<()>>,
    abort: fn(*mut Inner<()>)
}

impl Task {
    pub fn new<R>(fun: R, ptr: Option<*mut Inner<R::Output>>) -> Self
    where
        R: Runnable
    {
        Self {
            fun: TaskFun::new(move || {
                let res = catch_unwind(AssertUnwindSafe(|| fun.run()))
                    .map_err(Into::into);
                ptr.map(move |ptr| unsafe {
                    set_result(ptr, res);
                });
            }),
            inner: ptr.map(|inner| inner as *mut Inner<()>),
            abort: abort_task::<R::Output>
        }
    }

    pub fn abort(mut self) {
        self.inner.take().map(|ptr| (self.abort)(ptr));
    }

    pub fn run(self) {
        self.fun.call();
    }
}

unsafe fn set_result<T>(ptr: *mut Inner<T>, result: Result<T>) {
    if ptr.is_null() {
        return;
    }
    let inner = &mut *ptr;
    inner.data = Some(result);
    inner.notifier.take().map(|notifier| notifier.notify());
}

fn abort_task<T>(ptr: *mut Inner<()>) {
    unsafe { set_result(ptr as *mut Inner<T>, Err(Error::Aborted)); }
}
