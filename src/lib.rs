#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::cargo)]
#![no_std]

#[cfg(test)]
extern crate std;

use core::mem::{size_of, ManuallyDrop};
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};

/// A [`Box`]-like type that uses `mlock` to prevent paging the allocated memory
/// to disk.
///
/// **Note: This type allocates in multiples of the operating system's page
/// size. This could lead to more memory usage than expected if many instances
/// of this type are used.**
pub struct LockedBox<T>(NonNull<T>);

impl<T> LockedBox<T> {
    /// Creates a new locked box with `contained` in a newly allocated,
    /// `mlock`-protected region of memory.
    ///
    /// # Panics
    ///
    /// This function panics If `size_of::<T>() >= usize::MAX - 4 * PAGE_SIZE`.
    pub fn new(contained: T) -> Self {
        // SAFETY: no references are made to the data contained by the allocated
        // memory until after `contained` as been written. The size of the
        // allocation is checked by `memsec`.
        let memory = unsafe {
            let memory = memsec::malloc::<T>().expect("allocation too large");
            // It is important to lock the memory before storing the value,
            // otherwise the process could be preempted between the write and
            // the mlock calls, and the memory theoretically could be paged to
            // disk during this preemption. By locking before writing, we ensure
            // `contained` is not paged to disk. The stack, however, could still
            // be paged, but that is a problem for another crate to solve.
            memsec::mlock(memory.as_ptr().cast(), size_of::<T>());
            ptr::write(memory.as_ptr(), contained);
            memory
        };
        Self(memory)
    }

    /// Returns the pointer to the underlying data.
    #[must_use]
    pub const fn ptr(boxed: &LockedBox<T>) -> *mut T {
        boxed.0.as_ptr()
    }

    /// Extracts the contained value from `boxed`.
    #[must_use]
    pub fn unbox(boxed: LockedBox<T>) -> T {
        // Prevent our `drop` implementation from double-dropping the contained value.
        let boxed = ManuallyDrop::new(boxed);
        unsafe {
            let contained = ptr::read(boxed.0.as_ptr());
            memsec::free(boxed.0);
            contained
        }
    }
}

impl<T> Deref for LockedBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The borrow checker prevents any invalid reference
        // attempts since the `NonNull` is owned by `LockedBox` and never
        // exposed.
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for LockedBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The borrow checker prevents any invalid reference
        // attempts since the `NonNull` is owned by `LockedBox` and never
        // exposed.
        unsafe { self.0.as_mut() }
    }
}

impl<T> Drop for LockedBox<T> {
    fn drop(&mut self) {
        // SAFETY: The `NonNull` is exclusively owned by us, and `unbox` uses
        // ManuallyDrop to ensure this code isn't executed during that function.
        unsafe {
            ptr::drop_in_place(self.0.as_ptr());
            memsec::free(self.0);
        }
    }
}

#[test]
fn doesnt_crash() {
    let locked = LockedBox::new(1_u8);
    assert_eq!(*locked, 1);
}

#[test]
fn drops_correctly() {
    use std::{cell::RefCell, rc::Rc};

    #[derive(Default)]
    struct Droppable(Rc<RefCell<bool>>);

    impl Drop for Droppable {
        fn drop(&mut self) {
            let mut dropped = (*self.0).borrow_mut();
            assert!(!*dropped, "already dropped");
            *dropped = true;
        }
    }

    // Verify drop is called exactly once when dropping.
    let locked = LockedBox::new(Droppable::default());
    let dropped = (*locked).0.clone();
    drop(locked);
    let mut dropped = Rc::try_unwrap(dropped).expect("Rc has clones");
    assert!(*dropped.get_mut());

    // Verify drop is called exactly once when unboxing
    let locked = LockedBox::new(Droppable::default());
    let unboxed = LockedBox::unbox(locked);
    drop(unboxed);
}

#[test]
fn allows_zero_sized() {
    LockedBox::new(());
}
