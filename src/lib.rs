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

/// Spawns a new task into the pool, returning a [`handle`] that can be used to retrieve the output.
///
/// [`handle`]: crate::join::JoinHandle
pub fn spawn<R>(runnable: R) -> JoinHandle<R::Output>
where
    R: Runnable
{
    context::get().spawn(runnable)
}

/// Like [`spawn`], spawns a new task into the pool, but doesn't return a handle, so the output
/// cannot be retrieved and the allocation needed to do so is skipped.
///
/// [`spawn`]: crate::spawn
pub fn spawn_detached<R>(runnable: R)
where
    R: Runnable
{
    context::get().spawn_detached(runnable)
}

/// Spawns a new task that will be executed periodically by the thread pool every specified time
/// and the specified amount of times.
pub fn spawn_periodic<T>(task: T, every: Duration, times: Option<usize>)
where
    T: Fn() + Send + 'static
{
    context::get().spawn_periodic(task, every, times);
}
