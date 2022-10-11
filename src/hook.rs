use tiny_fn::tiny_fn;

tiny_fn! {
    pub struct HookFn = Fn();
}

#[derive(Default)]
pub struct Hooks {
    pub on_start: Option<HookFn<'static>>,
    pub on_stop: Option<HookFn<'static>>,
    pub after_task: Option<HookFn<'static>>,
    pub before_task: Option<HookFn<'static>>
}
