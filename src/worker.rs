use std::sync::Arc;
use std::time::Duration;
use crate::core::Core;
use crate::handle::Handle;

pub struct Worker {
    core: Arc<Core>
}

impl Worker {
    pub fn new(core: Arc<Core>) -> Self {
        Self {
            core
        }
    }

    pub fn run(self) {
        crate::context::set(Handle { core: Arc::clone(&self.core) });

        self.core.hooks.on_start.as_ref().map(|fun| fun.call());

        while !self.core.exit {
            let timeout = self.try_schedule_periodical();
            if self.core.driver.queue.is_empty() {
                let mut lock = self.core.mutex.lock();

                if timeout {
                    self.core.condvar.wait_for(&mut lock, Duration::from_millis(150));
                } else {
                    self.core.condvar.wait(&mut lock);
                }
            }
            if let Some(task) = self.core.driver.queue.pop() {
                task.run();
            }
        }

        self.core.hooks.on_stop.as_ref().map(|fun| fun.call());
        crate::context::clear();
    }

    fn try_schedule_periodical(&self) -> bool {
        if let Some(mut lock) = self.core.timer.try_lock() {
            lock.schedule_available(&self.core.driver.queue);
            true
        } else {
            false
        }
    }
}
