use std::fmt;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

/// An error indicating that the vector is at full capacity.
#[derive(Debug, PartialEq, Eq)]
pub struct CapacityError<T>(pub T);

impl<T> fmt::Display for CapacityError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArrayVec capacity reached")
    }
}

impl<T: fmt::Debug> std::error::Error for CapacityError<T> {}

/// A fixed-capacity, stack-allocated vector using const generics.
///
/// Stores elements inline within uninitialized memory (`MaybeUninit<T>`),
/// requiring zero heap allocations and no `T: Default` or `T: Copy` bounds.
pub struct ArrayVec<T, const CAP: usize> {
    data: [MaybeUninit<T>; CAP],
    len: usize,
}

impl<T, const CAP: usize> ArrayVec<T, CAP> {
    /// Creates a new, empty `ArrayVec`.
    #[inline]
    pub fn new() -> Self {
        Self {
            // SAFETY: An uninitialized array of `MaybeUninit<T>` is valid to construct
            // because `MaybeUninit` does not require initialization.
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    /// Returns the number of initialized elements currently in the vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the maximum capacity of the vector.
    #[inline]
    pub const fn capacity(&self) -> usize {
        CAP
    }

    /// Returns `true` if the vector is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == CAP
    }

    /// Appends an element to the back of the vector.
    ///
    /// If the vector is full, returns `Err(CapacityError(element))`, giving back ownership.
    pub fn push(&mut self, element: T) -> Result<(), CapacityError<T>> {
        if self.len >= CAP {
            return Err(CapacityError(element));
        }

        // SAFETY: self.len < CAP, so self.data[self.len] is a valid slot within the array bounds.
        self.data[self.len].write(element);
        self.len += 1;
        Ok(())
    }

    /// Removes the last element from the vector and returns it, or `None` if it is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        // SAFETY: The element at `self.len` was previously initialized and has not been read yet.
        // Decrementing `self.len` ensures it cannot be accessed or double-dropped.
        unsafe { Some(self.data[self.len].assume_init_read()) }
    }

    /// Removes all elements from the vector, dropping each initialized item.
    pub fn clear(&mut self) {
        // Drop elements in reverse order of initialization (standard Vec behavior).
        while self.len > 0 {
            self.len -= 1;
            // SAFETY: Elements at 0..len were initialized.
            unsafe {
                self.data[self.len].assume_init_drop();
            }
        }
    }

    /// Extracts a slice containing the initialized elements.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: data[0..len] are guaranteed to be valid, initialized instances of T.
        unsafe {
            std::slice::from_raw_parts(self.data.as_ptr() as *const T, self.len)
        }
    }

    /// Extracts a mutable slice containing the initialized elements.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: data[0..len] are guaranteed to be valid, initialized instances of T.
        unsafe {
            std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.len)
        }
    }
}

impl<T, const CAP: usize> Default for ArrayVec<T, CAP> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAP: usize> Deref for ArrayVec<T, CAP> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const CAP: usize> DerefMut for ArrayVec<T, CAP> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const CAP: usize> Drop for ArrayVec<T, CAP> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T: fmt::Debug, const CAP: usize> fmt::Debug for ArrayVec<T, CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), f)
    }
}

impl<T: PartialEq, const CAP: usize> PartialEq for ArrayVec<T, CAP> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq, const CAP: usize> Eq for ArrayVec<T, CAP> {}

/// An owning iterator for `ArrayVec`.
pub struct IntoIter<T, const CAP: usize> {
    vec: ArrayVec<T, CAP>,
    index: usize,
}

impl<T, const CAP: usize> Iterator for IntoIter<T, CAP> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            let item = unsafe { self.vec.data[self.index].assume_init_read() };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<T, const CAP: usize> ExactSizeIterator for IntoIter<T, CAP> {}

impl<T, const CAP: usize> Drop for IntoIter<T, CAP> {
    fn drop(&mut self) {
        // Drop any remaining unconsumed elements
        for i in self.index..self.vec.len {
            unsafe {
                self.vec.data[i].assume_init_drop();
            }
        }
        // Set len to 0 so the inner ArrayVec::drop doesn't attempt to re-drop already read/dropped items
        self.vec.len = 0;
    }
}

impl<T, const CAP: usize> IntoIterator for ArrayVec<T, CAP> {
    type Item = T;
    type IntoIter = IntoIter<T, CAP>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            vec: self,
            index: 0,
        }
    }
}

impl<'a, T, const CAP: usize> IntoIterator for &'a ArrayVec<T, CAP> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, T, const CAP: usize> IntoIterator for &'a mut ArrayVec<T, CAP> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}
