# Custom Iterator Combinators (`MyMap`, `MyFilter`)

A lightweight, zero-cost implementation of lazy iterator adapters and an extension trait in Rust, built from scratch without utilizing the standard library's `.map()` or `.filter()`.

---

## Features

- **`MyMap<I, F>`**: Lazy element transformation adapter with `FnMut(I::Item) -> B`.
- **`MyFilter<I, P>`**: Lazy predicate filter adapter with `FnMut(&I::Item) -> bool`.
- **`CustomIteratorExt`**: Ergonomic extension trait blanket-implemented for all `I: Iterator`, allowing fluent method chaining (`.my_map()`, `.my_filter()`).
- **Strict Laziness**: No evaluation occurs upon instantiation or chaining—evaluations only happen per-item when `.next()` is called.
- **Accurate `size_hint`**: Propagates upstream bounds and adapts lower/upper bounds for filtering.

---

## Example Usage

```rust
use custom_iterators::CustomIteratorExt;

fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let result: Vec<i32> = numbers
        .into_iter()
        .my_filter(|x| x % 2 != 0) // Keep odd numbers: [1, 3, 5, 7, 9]
        .my_map(|x| x * 10)         // Multiply by 10: [10, 30, 50, 70, 90]
        .my_filter(|x| *x > 40)    // Keep > 40: [50, 70, 90]
        .collect();

    assert_eq!(result, vec![50, 70, 90]);
}
```

---

## Running Tests

```bash
cargo test
```
