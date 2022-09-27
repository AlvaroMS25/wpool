use std::cell::RefCell;
use crate::handle::Handle;

thread_local! {
    static HANDLE: RefCell<Option<Handle>> = RefCell::new(None);
}

pub fn get() -> Handle {
    try_get().unwrap()
}

pub fn try_get() -> Option<Handle> {
    HANDLE.try_with(|cell| {
        cell.try_borrow().ok()?.clone()
    }).ok()?
}

pub fn set(handle: Handle) {
    HANDLE.with(|cell| {
        *cell.borrow_mut() = Some(handle);
    })
}

pub fn clear() {
    HANDLE.with(|inner| {
        *inner.borrow_mut() = None;
    })
}
