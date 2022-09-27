use crate::sync::Task;
use crossbeam_queue::SegQueue;
use crate::periodic::PeriodicTask;

pub enum Either<A, B> {
    Left(A),
    Right(B)
}

impl Either<Task, PeriodicTask> {
    pub fn run(self) {
        match self {
            Self::Left(task) => task.run(),
            Self::Right(task) => task.run()
        }
    }
}

#[derive(Default)]
pub struct Driver {
    pub queue: SegQueue<Either<Task, PeriodicTask>>,
}

impl Driver {
    pub fn schedule(&self, task: Either<Task, PeriodicTask>) {
        self.queue.push(task);
    }

    pub fn clear(&self) {
        while let Some(item) = self.queue.pop() {
            match item {
                Either::Left(task) => task.abort(),
                _ => ()
            }
        }
    }

}
