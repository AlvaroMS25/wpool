use std::thread::JoinHandle;
use crate::driver::{Driver, Either};
use crate::hook::Hooks;
use crate::timer::Timer;
use parking_lot::{Condvar, Mutex};
use crate::periodic::PeriodicTask;
use crate::sync::Task;

#[derive(Default)]
pub struct Core {
    pub driver: Driver,
    pub hooks: Hooks,
    pub timer: Mutex<Timer>,
    pub mutex: Mutex<()>,
    pub condvar: Condvar,
    pub handles: Mutex<Vec<JoinHandle<()>>>,
    pub exit: bool
}

impl Core {
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

    pub fn shutdown(&self) {
        self.driver.clear();
        crate::context::clear();
        let mut lock = self.handles.lock();

        unsafe {
            let this = &mut *(self as *const Self as *mut Self);
            this.exit = true;
        }

        self.condvar.notify_all();

        lock.drain(..).for_each(|handle| {
            let _ = handle.join();
        });
    }
}

unsafe impl Send for Core {}
unsafe impl Sync for Core {}
