# Typestate Pattern & Phantom Types (Order Flow)

An idiomatic Rust implementation of the **Typestate Pattern** that encodes an e-commerce order lifecycle state machine directly into Rust's type system using **Zero-Sized Types (ZSTs)**, `std::marker::PhantomData`, and **move semantics**.

## Key Concepts

- **Zero-Sized Types (ZSTs)**: State markers (`Draft`, `Paid`, `Shipped`, `Delivered`, `Cancelled`) consume 0 bytes of memory (`std::mem::size_of::<State>() == 0`).
- **`PhantomData<State>`**: Allows `Order<State>` to be generic over its state without holding runtime state values or incurring memory layout overhead.
- **Move Semantics (`self`)**: Transitions consume the order instance by value, eliminating stale state reuse or invalid aliasing at compile time.
- **Compile-Time Guarantees**: Invalid state transitions (such as shipping an unpaid draft, modifying a paid order, or double payments) fail at compile time.

## State Transition Diagram

```text
                  +-----------------------------------+
                  |                                   |
                  v                                   |
 [*] ---> [ Draft ] --(pay)--> [ Paid ] --(ship)--> [ Shipped ] --(deliver)--> [ Delivered ] ---> [*]
             |                   |
          (cancel)            (cancel)
             |                   |
             +---------> [ Cancelled ] ---> [*]
```

## Transition Matrix & Compile-Time Enforcement

| Method | `Order<Draft>` | `Order<Paid>` | `Order<Shipped>` | `Order<Delivered>` | `Order<Cancelled>` |
| :--- | :---: | :---: | :---: | :---: | :---: |
| `add_item()` | Allowed | **Compile Error** | **Compile Error** | **Compile Error** | **Compile Error** |
| `pay()` | Allowed | **Compile Error** | **Compile Error** | **Compile Error** | **Compile Error** |
| `ship()` | **Compile Error** | Allowed | **Compile Error** | **Compile Error** | **Compile Error** |
| `deliver()` | **Compile Error** | **Compile Error** | Allowed | **Compile Error** | **Compile Error** |
| `cancel()` | Allowed | Allowed | **Compile Error** | **Compile Error** | **Compile Error** |

## Project Structure

```text
typestate_order/
├── src/
│   ├── lib.rs       # Core typestate Order<S>, PhantomData, transitions & doc-tests (129 lines)
│   └── main.rs      # Interactive demo & memory footprint verification
├── tests/
│   └── order_tests.rs # Invariant, lifecycle, and ZST memory size tests
└── Cargo.toml
```

## Running Tests & Verification

### 1. Run Unit Tests & Doc-tests (including `compile_fail` checks)

```bash
cargo test
```

Verifies:

- Full lifecycle: `Draft` $\to$ `Paid` $\to$ `Shipped` $\to$ `Delivered`
- Empty order payment guards
- Cancellation and refund paths
- Zero-cost memory assertions (`size_of == 0`)
- Negative `compile_fail` assertions (compiler rejects invalid transitions)

### 2. Run Demo Application

```bash
cargo run
```
