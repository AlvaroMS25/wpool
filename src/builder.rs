use tiny_fn::tiny_fn;
use crate::handle::Handle;
use crate::hook::{Hooks, HookFn};
use std::io;
use std::sync::Arc;
use std::thread;
use crate::core::Core;
use crate::worker::Worker;

tiny_fn! {
    pub(crate) struct NameFn = Fn() -> String;
}

/// A builder used to create a new worker pool.
///
/// The pool uses by default the double of threads physical cores the CPU has.
pub struct WorkerPoolBuilder {
    threads: usize,
    stack_size: Option<usize>,
    name: NameFn<'static>,
    hooks: Hooks
}

impl WorkerPoolBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            threads: num_cpus::get_physical() * 2,
            stack_size: None,
            name: NameFn::new(|| String::from("Worker-Pool worker")),
            hooks: Hooks::default()
        }
    }

    /// Sets the number of threads to use.
    pub fn threads(&mut self, threads: usize) -> &mut Self {
        self.threads = threads;
        self
    }

    /// Sets the name of the threads of the worker pool.
    pub fn set_name(&mut self, name: impl ToString) -> &mut Self {
        let name = name.to_string();
        self.name = NameFn::new(move || name.clone());

        self
    }

    /// Sets a function used to determine the name of the worker threads.
    pub fn set_name_fn<F>(&mut self, fun: F) -> &mut Self
    where
        F: Fn() -> String + Send + 'static
    {
        self.name = NameFn::new(fun);
        self
    }

    /// Sets a function to execute at the start of each thread.
    pub fn on_start<F>(&mut self, fun: F) -> &mut Self
    where
        F: Fn() + Send + 'static
    {
        self.hooks.on_start = Some(HookFn::new(fun));
        self
    }

    /// Sets a function to execute before stopping each thread.
    pub fn on_stop<F>(&mut self, fun: F) -> &mut Self
    where
        F: Fn() + Send + 'static
    {
        self.hooks.on_stop = Some(HookFn::new(fun));
        self
    }

    /// Builds and starts the pool consuming the builder.
    pub fn launch_owned(self) -> io::Result<Handle> {
        let mut handles = Vec::new();
        let core = Arc::new(Core::new(self.hooks));

        for _ in 0..self.threads {
            let mut builder = thread::Builder::new()
                .name(self.name.call());
            if let Some(size) = self.stack_size {
                builder = builder.stack_size(size);
            }

            let core = Arc::clone(&core);
            handles.push(builder.spawn(|| Worker::new(core).run())?);
        }

        *core.handles.lock() = handles;

        crate::context::set(Handle { core: Arc::clone(&core) });

        Ok(Handle { core })
    }

    /// Builds and starts the pool without taking ownership of the builder
    pub fn launch(&mut self) -> io::Result<Handle> {
        let this = std::mem::replace(self, Self::new());
        this.launch_owned()
    }

}
