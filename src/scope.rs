use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool};
use parking_lot::Mutex;
use crate::handle::Handle;
use crate::join::{JoinHandle, ScopedJoinHandle};
use crate::Runnable;

pub struct Scope<'scope> {
    handle: &'scope Handle,
    running_tasks: AtomicUsize,
}

impl<'scope> Scope<'scope> {
    pub fn spawn<F, R>(&'scope self, fun: F) -> ScopedJoinHandle<'scope, R>
    where
        F: FnOnce() -> R + Send + 'scope,
        R: Send + 'scope
    {
        let cell = Arc::new(UnsafeCell::new(None));
        let cell_clone = Arc::clone(&cell);

        let boxed = Box::new(move || {
            let result = fun();
            unsafe {
                *&mut *cell_clone.get() = Some(result);
            }
        }) as Box<dyn FnOnce() + 'scope>;

        let transmuted = unsafe {
            std::mem::transmute::<Box<dyn FnOnce() + 'scope>, Box<dyn FnOnce() + Send + 'static>>(boxed)
        };

        ScopedJoinHandle {
            join: self.handle.spawn(transmuted),
            scope: self,
            cell
        }
    }
}
