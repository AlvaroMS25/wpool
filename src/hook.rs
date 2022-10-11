use tiny_fn::tiny_fn;

tiny_fn! {
    pub struct HookFn = Fn();
}

/// A container for all the hooks provided to the pool.
#[derive(Default)]
pub struct Hooks {
    /// The function to execute before a thread starts doing work.
    pub on_start: Option<HookFn<'static>>,
    /// The function to execute before a thread stops.
    pub on_stop: Option<HookFn<'static>>,
    /// The function to execute after a task is executed.
    pub after_task: Option<HookFn<'static>>,
    /// The function to execute before executing a task.
    pub before_task: Option<HookFn<'static>>
}
