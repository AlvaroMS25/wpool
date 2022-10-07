/// A trait defining what can be spawned using [`spawn`]/[`spawn_detached`].
///
/// [`spawn`]: crate::spawn
/// [`spawn_detached`]: crate::spawn_detached
pub trait Runnable: Send + 'static {
    /// The type of the value the task returns when executed.
    type Output: Sized + 'static;
    /// The task's function body.
    fn run(self) -> Self::Output;
}

impl<Fun, Ret> Runnable for Fun
where
    Fun: FnOnce() -> Ret + Send + 'static,
    Ret: Sized + 'static
{
    type Output = Ret;

    fn run(self) -> Self::Output {
        (self)()
    }
}
