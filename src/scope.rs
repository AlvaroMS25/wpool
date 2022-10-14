use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crossbeam_utils::sync::Unparker;
use crate::handle::Handle;
use crate::join::ScopedJoinHandle;

pub struct Scope<'scope> {
    pub(crate) handle: &'scope Handle,
    pub(crate) inner: Arc<ScopeInner>,
}

pub struct ScopeInner {
    pub running_tasks: AtomicUsize,
    pub unparker: Unparker,
}

impl ScopeInner {
    pub fn increment_task_number(&self) {
        if self.running_tasks.fetch_add(1, Ordering::Relaxed) > usize::MAX / 2 {
            self.decrement_task_number();
            panic!("Too many tasks spawned");
        }
    }

    pub fn decrement_task_number(&self) {
        if self.running_tasks.fetch_sub(1, Ordering::Release) == 1 {
            self.unparker.unpark();
        }
    }
}

impl<'scope> Scope<'scope> {
    pub fn new(handle: &'scope Handle, unparker: Unparker) -> Self {
        Self {
            handle,
            inner: Arc::new(ScopeInner {
                running_tasks: AtomicUsize::new(0),
                unparker,
            }),
        }
    }

    pub fn spawn<F, R>(&'scope self, fun: F) -> ScopedJoinHandle<'scope, R>
    where
        F: FnOnce() -> R + Send + 'scope,
        R: Send + 'scope
    {
        let cell = Arc::new(UnsafeCell::new(None));
        let cell_clone = Arc::clone(&cell);
        let inner = Arc::clone(&self.inner);

        self.inner.increment_task_number();

        let boxed = Box::new(move || {
            let result = fun();
            unsafe {
                *&mut *cell_clone.get() = Some(result);
            }
            inner.decrement_task_number();
        }) as Box<dyn FnOnce() + 'scope>;

        let transmuted = unsafe {
            std::mem::transmute::<Box<dyn FnOnce() + 'scope>, Box<dyn FnOnce() + Send + 'static>>(boxed)
        };

        ScopedJoinHandle {
            join: self.handle.spawn(transmuted),
            cell,
            _marker: PhantomData
        }
    }
}
