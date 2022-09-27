pub trait Runnable: Send + 'static {
    type Output: Sized + 'static;
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
