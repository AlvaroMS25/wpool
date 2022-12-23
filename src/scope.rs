use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crossbeam_queue::SegQueue;
use crossbeam_utils::sync::{Parker, Unparker};
use parking_lot::Mutex;
use crate::handle::Handle;
use crate::join::ScopedJoinHandle;
use crate::JoinHandle;

pub struct Scope<'scope, 'env: 'scope> {
    pub(crate) handle: &'scope Handle,
    pub(crate) inner: Arc<ScopeInner>,
    pub(crate) parker: Parker,
    _marker: PhantomData<&'env ()>
}

pub struct ScopeInner {
    pub running_tasks: AtomicUsize,
    dropped_handles: SegQueue<JoinHandle<()>>,
    unparker: Unparker
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

impl<'scope, 'env: 'scope> Scope<'scope, 'env> {
    pub fn new(handle: &'scope Handle) -> Self {
        let parker = Parker::new();
        Self {
            handle,
            inner: Arc::new(ScopeInner {
                running_tasks: AtomicUsize::new(0),
                dropped_handles: SegQueue::new(),
                unparker: parker.unparker().clone()
            }),
            parker,
            _marker: PhantomData
        }
    }

    pub fn spawn<F, R>(&'scope self, fun: F) -> ScopedJoinHandle<'scope, 'env, R>
    where
        F: FnOnce() -> R + Send + 'scope,
        R: Send + 'scope
    {
        let mutex = Arc::new(Mutex::new(None));
        let mutex_clone = Arc::clone(&mutex);
        let inner = Arc::clone(&self.inner);

        self.inner.increment_task_number();

        let boxed = Box::new(move || {
            let result = fun();
            *mutex_clone.lock() = Some(result);
            inner.decrement_task_number();
        }) as Box<dyn FnOnce() + 'scope>;

        let transmuted = unsafe {
            std::mem::transmute::<Box<dyn FnOnce() + 'scope>, Box<dyn FnOnce() + Send + 'static>>(boxed)
        };

        ScopedJoinHandle {
            join: Some(self.handle.spawn(transmuted)),
            mutex: Some(mutex),
            _marker: PhantomData
        }
    }

    pub(crate) fn wait(&self) {
        while self.inner.running_tasks.load(Ordering::Acquire) != 0 {
            self.parker.park();
        }
    }
}
