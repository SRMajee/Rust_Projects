//! Custom Lazy Iterator Combinators
//!
//! This module implements custom iterator combinators (`MyMap` and `MyFilter`)
//! along with an extension trait (`CustomIteratorExt`) to allow fluent chaining.
//! Everything is evaluated lazily: no computation happens until `next()` is polled.

/// An iterator adapter that applies a function to each element.
pub struct MyMap<I, F> {
    iter: I,
    f: F,
}

impl<I, F> MyMap<I, F> {
    /// Creates a new `MyMap` adapter.
    pub fn new(iter: I, f: F) -> Self {
        Self { iter, f }
    }
}

impl<B, I, F> Iterator for MyMap<I, F>
where
    I: Iterator,
    F: FnMut(I::Item) -> B,
{
    type Item = B;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(&mut self.f)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// An iterator adapter that filters elements based on a predicate.
pub struct MyFilter<I, P> {
    iter: I,
    predicate: P,
}

impl<I, P> MyFilter<I, P> {
    /// Creates a new `MyFilter` adapter.
    pub fn new(iter: I, predicate: P) -> Self {
        Self { iter, predicate }
    }
}

impl<I, P> Iterator for MyFilter<I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.iter.next() {
            if (self.predicate)(&item) {
                return Some(item);
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        // A filter may yield anywhere between 0 and upper elements
        (0, upper)
    }
}

/// Extension trait enabling ergonomic chaining of custom combinators.
pub trait CustomIteratorExt: Iterator {
    /// Applies a closure to each element, producing a new lazy iterator.
    fn my_map<B, F>(self, f: F) -> MyMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B,
    {
        MyMap::new(self, f)
    }

    /// Filters elements yielding only those for which `predicate` returns `true`.
    fn my_filter<P>(self, predicate: P) -> MyFilter<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        MyFilter::new(self, predicate)
    }
}

// Blanket implementation for all types implementing `Iterator`.
impl<I: Iterator> CustomIteratorExt for I {}
