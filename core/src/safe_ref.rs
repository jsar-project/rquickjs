#[cfg(not(feature = "parallel"))]
use core::cell::RefCell as Cell;

#[cfg(all(feature = "parallel", feature = "std"))]
use std::sync::Mutex as Cell;

#[cfg(all(feature = "parallel", not(feature = "std")))]
use spin::Mutex as Cell;

#[cfg(not(feature = "parallel"))]
pub use core::cell::RefMut as Lock;

#[cfg(not(feature = "parallel"))]
pub use alloc::rc::{Rc as Ref, Weak};

#[cfg(feature = "parallel")]
pub use alloc::sync::{Arc as Ref, Weak};

#[cfg(all(feature = "parallel", feature = "std"))]
pub use std::sync::MutexGuard as Lock;

#[cfg(all(feature = "parallel", not(feature = "std")))]
pub use spin::MutexGuard as Lock;

#[repr(transparent)]
#[derive(Debug)]
pub struct Mut<T: ?Sized>(Cell<T>);

impl<T> Mut<T> {
    pub fn new(inner: T) -> Self {
        Self(Cell::new(inner))
    }
}

impl<T: Default> Default for Mut<T> {
    fn default() -> Self {
        Mut::new(T::default())
    }
}

impl<T: ?Sized> Mut<T> {
    pub fn lock(&self) -> Lock<T> {
        #[cfg(not(feature = "parallel"))]
        {
            self.0.borrow_mut()
        }

        #[cfg(all(feature = "parallel", feature = "std"))]
        {
            self.0.lock().unwrap()
        }

        #[cfg(all(feature = "parallel", not(feature = "std")))]
        {
            self.0.lock()
        }
    }

    pub fn try_lock(&self) -> Option<Lock<T>> {
        #[cfg(not(feature = "parallel"))]
        {
            self.0.try_borrow_mut().ok()
        }

        #[cfg(all(feature = "parallel", feature = "std"))]
        {
            self.0.lock().ok()
        }

        #[cfg(all(feature = "parallel", not(feature = "std")))]
        {
            self.0.try_lock()
        }
    }
}
