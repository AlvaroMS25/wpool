#![allow(unstable_name_collisions)]

pub mod builder;
mod context;
mod core;
mod driver;
pub mod error;
pub mod handle;
mod hook;
pub mod join;
mod periodic;
pub mod runnable;
mod sync;
mod timer;
mod wait;
mod worker;

#[cfg(test)]
mod test;

use std::time::Duration;
use join::JoinHandle;
use runnable::Runnable;

pub fn spawn<R>(runnable: R) -> JoinHandle<R::Output>
where
    R: Runnable
{
    context::get().spawn(runnable)
}

pub fn spawn_detached<R>(runnable: R)
where
    R: Runnable
{
    context::get().spawn_detached(runnable)
}

pub fn spawn_periodic<T>(task: T, every: Duration, times: Option<usize>)
where
    T: Fn() + Send + 'static
{
    context::get().spawn_periodic(task, every, times);
}
