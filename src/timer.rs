use crossbeam_queue::SegQueue;
use crate::driver::Either;
use crate::periodic::PeriodicTask;
use crate::sync::Task;
use drain_filter_polyfill::VecExt;
use parking_lot::Condvar;

/// The queue of periodic task, the task here are scheduled at the main queue when needed.
#[derive(Default)]
pub struct Timer {
    pub waiting: Vec<PeriodicTask>,
}

impl Timer {
    pub fn schedule(&mut self, task: PeriodicTask) {
        self.waiting.push(task);
    }

    pub fn schedule_available(&mut self, cv: &Condvar, to: &SegQueue<Either<Task, PeriodicTask>>) {
        for task in self.waiting.drain_filter(|task| task.can_run()) {
            to.push(Either::Right(task));
            cv.notify_one();
        }
    }
}
