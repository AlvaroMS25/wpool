use crossbeam_queue::SegQueue;
use crate::driver::Either;
use crate::periodic::PeriodicTask;
use crate::sync::Task;
use drain_filter_polyfill::VecExt;

#[derive(Default)]
pub struct Timer {
    pub waiting: Vec<PeriodicTask>,
}

impl Timer {
    pub fn schedule(&mut self, task: PeriodicTask) {
        self.waiting.push(task);
    }

    pub fn schedule_available(&mut self, to: &SegQueue<Either<Task, PeriodicTask>>) {
        for task in self.waiting.drain_filter(|task| task.can_run()) {
            to.push(Either::Right(task));
        }
    }
}
