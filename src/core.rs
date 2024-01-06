use std::cell::UnsafeCell;
use std::ops::Deref;
use std::thread::JoinHandle;
use crate::driver::{Driver, Either};
use crate::hook::Hooks;
use crate::timer::Timer;
use parking_lot::{Condvar, Mutex};
use crate::periodic::PeriodicTask;
use crate::sync::Task;

pub struct Core(UnsafeCell<CoreInner>);

/// The core shared among all worker threads and handles.
#[derive(Default)]
pub struct CoreInner {
    /// The queue of tasks of the pool.
    pub driver: Driver,
    /// The hooks the pool has.
    pub hooks: Hooks,
    /// The timer used to store periodic tasks that are not ready to run.
    pub timer: Mutex<Timer>,
    /// A mutex used along with the condvar to put to sleep the threads.
    pub mutex: Mutex<()>,
    /// The condvar used along the mutex to put to sleep the threads.
    pub condvar: Condvar,
    /// The handles of the worker threads
    pub handles: Mutex<Vec<JoinHandle<()>>>,
    /// Whether the pool should exit or not.
    pub exit: bool
}

impl Core {
    pub fn new(hooks: Hooks) -> Self {
        Self(UnsafeCell::new(CoreInner::new(hooks)))
    }

    pub fn inner_mut(&self) -> &mut <Self as Deref>::Target {
        unsafe { &mut *self.0.get() }
    }
}

impl Deref for Core {
    type Target = CoreInner;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}

impl CoreInner {
    pub fn new(hooks: Hooks) -> Self {
        Self {
            hooks,
            ..Default::default()
        }
    }

    fn assert_running(&self) {
        if self.exit {
            panic!("Threadpool not running");
        }
    }

    pub fn schedule(&self, task: Task) {
        self.assert_running();
        self.driver.schedule(Either::Left(task));
        self.condvar.notify_one();
    }

    pub fn schedule_periodical(&self, task: PeriodicTask) {
        self.assert_running();
        let notify = task.can_run();
        self.timer.lock().schedule(task);
        if notify {
            self.condvar.notify_one();
        }
    }

    pub fn shutdown(&mut self) {
        self.assert_running();
        self.driver.clear();
        crate::context::clear();
        let mut lock = self.handles.lock();

        self.exit = true;

        self.condvar.notify_all();

        lock.drain(..).for_each(|handle| {
            let _ = handle.join();
        });
    }
}

unsafe impl Send for Core {}
unsafe impl Sync for Core {}
