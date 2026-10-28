# The Rust Behind qsengine

This guide covers every Rust language feature, standard-library function and crate function used in qsengine. The most important ones come first: once you understand the first ten sections, you can read the whole engine. Each section explains what the item is, why the engine uses it, and its syntax, with a short example taken from the codebase.

Files referenced: `src/lib.rs`, `src/backend.rs`, `src/engine.rs`, `src/gates.rs`, `src/gpu.rs`, `src/shaders/apply_gate.wgsl`, `src/circuit.rs`, `src/python.rs`, `src/main.rs`.

---

## Contents

**Part 1: Core language (you need all of these)**
1. [Structs](#1-structs)
2. [`impl` blocks and methods](#2-impl-blocks-and-methods)
3. [Traits](#3-traits)
4. [`Vec<T>` and `vec!`](#4-vect-and-vec)
5. [Complex numbers (`num_complex::Complex64`)](#5-complex-numbers-num_complexcomplex64)
6. [Bitwise operators and bit masks](#6-bitwise-operators-and-bit-masks)
7. [Ownership, borrowing, `&` and `&mut`](#7-ownership-borrowing--and-mut)
8. [Slices: `chunks_mut` and `split_at_mut`](#8-slices-chunks_mut-and-split_at_mut)
9. [Iterators: `iter`, `iter_mut`, `zip`, `enumerate`, `map`, `collect`](#9-iterators)
10. [Closures](#10-closures)

**Part 2: Errors and API design**

11. [`Result`, `Ok`, `Err` and the `?` operator](#11-result-ok-err-and-the--operator)
12. [Enums and `match`](#12-enums-and-match)
13. [Implementing `Display` and `Error`](#13-implementing-display-and-error)
14. [Trait objects: `dyn Trait` and `Box<dyn Trait>`](#14-trait-objects-dyn-trait-and-boxdyn-trait)
15. [Modules, `pub` and `use`](#15-modules-pub-and-use)
16. [Type aliases](#16-type-aliases)
17. [`const` and `const fn`](#17-const-and-const-fn)

**Part 3: Performance**

18. [Rayon: `par_iter_mut`, `par_chunks_mut`, `with_min_len`, `for_each`](#18-rayon-parallel-iterators)
19. [Iterator extras: `scan`, `fold`, `filter`, `count`](#19-iterator-extras-scan-fold-filter-count)
20. [`partition_point` (binary search)](#20-partition_point-binary-search)
21. [`fill` and `clone`](#21-fill-and-clone)
22. [Integer helpers: `div_ceil`, `next_multiple_of`, `leading_zeros`, `min`, `max`](#22-integer-helpers)
23. [`f64` maths: `sin_cos`, `consts`](#23-f64-maths)

**Part 4: Randomness and collections**

24. [`rand`: `rng()`, `random()`, `RngCore`, `SeedableRng`](#24-rand)
25. [`HashMap` and the `entry` API](#25-hashmap-and-the-entry-api)

**Part 5: GPU and concurrency**

26. [`OnceLock`: one-time global initialisation](#26-oncelock)
27. [`Mutex` and `get_mut`](#27-mutex-and-get_mut)
28. [`async`/`.await` and `pollster::block_on`](#28-async-await-and-pollsterblock_on)
29. [`#[repr(C)]` and `bytemuck`](#29-reprc-and-bytemuck)
30. [The wgpu functions](#30-the-wgpu-functions)
31. [WGSL (the shader language)](#31-wgsl)

**Part 6: Attributes, macros and tooling**

32. [`#[derive(...)]`](#32-derive)
33. [Cargo features and `#[cfg(...)]`](#33-cargo-features-and-cfg)
34. [`format!`, `println!`, `write!`](#34-format-println-write)
35. [Tests: `#[test]`, `assert!`, `assert_eq!`, `matches!`](#35-tests)
36. [Doc comments: `///` and `//!`](#36-doc-comments)
37. [`Instant` and `black_box` (benchmarking)](#37-instant-and-black_box)

**Part 7: Python bindings (PyO3)**

38. [`#[pyclass]` and `#[pymethods]`](#38-pyclass-and-pymethods)
39. [`#[new]`, `#[getter]` and dunder methods](#39-new-getter-and-dunder-methods)
40. [`PyResult` and converting errors with `From`](#40-pyresult-and-converting-errors-with-from)
41. [`#[pyo3(signature = ...)]` and `Option<T>`](#41-pyo3signature--and-optiont)
42. [`py.detach`: releasing the GIL](#42-pydetach-releasing-the-gil)
43. [`#[pymodule]` and `Bound`](#43-pymodule-and-bound)
44. [maturin, `cdylib` and abi3](#44-maturin-cdylib-and-abi3)

---

# Part 1: Core language

## 1. Structs

A `struct` groups related data under one name, like a class without inheritance. qsengine stores a quantum register as a struct holding the number of qubits and the vector of amplitudes. Each field has a name and a type, and `pub` makes a field readable from outside the module.

A struct says nothing about behaviour; methods are added separately with `impl` (section 2). Structs are laid out in memory with no hidden overhead, which matters when a state vector holds millions of entries.

```rust
// Definition
pub struct QuantumState {
    pub num_qubits: usize,          // field: type
    pub amplitudes: Vec<Complex64>,
}

// Construction (field-init shorthand when variable name == field name)
let s = QuantumState { num_qubits, amplitudes };

// Access
s.num_qubits
```

## 2. `impl` blocks and methods

An `impl` block attaches functions to a type. If a function's first parameter is `self`, `&self` or `&mut self`, it's a **method** and is called with dot syntax. Without a `self` parameter it's an **associated function** (like a static method), called with `Type::name()`. By convention, `new` is the constructor.

- `&self` borrows the value to read it.
- `&mut self` borrows it to change it.
- `self` takes ownership and consumes it.

`Self` (capital S) is shorthand for the type being implemented.

```rust
impl QuantumState {
    // associated function: QuantumState::new(3)
    pub fn new(num_qubits: usize) -> Result<Self> { ... }

    // method that reads: state.probability_of_index(0)
    pub fn probability_of_index(&self, index: usize) -> f64 {
        self.amplitudes[index].norm_sqr()
    }
}
```

## 3. Traits

A **trait** is a set of methods that a type promises to provide, like an interface in Java or an abstract base class in Python. It's the main tool behind qsengine's modular design. `QuantumBackend` defines the simulator API once. The CPU and GPU backends each implement four *required* methods and get every other method for free as *provided* (default) methods.

To add a new backend later (for example, a distributed one), you implement those four methods and everything else works: `h`, `cx`, `measure`, `sample` and validation. Provided methods can call required ones through `self`.

```rust
pub trait QuantumBackend {
    fn num_qubits(&self) -> usize;                    // required: no body
    fn apply_gate_kernel(&mut self, mask: usize, target: usize, gate: &Gate);

    fn h(&mut self, q: usize) -> Result<()> {         // provided: has a body
        self.apply_gate(q, &gates::H)
    }
}

// Implementing it for a type
impl QuantumBackend for QuantumState {
    fn num_qubits(&self) -> usize { self.num_qubits }
    fn apply_gate_kernel(&mut self, mask: usize, target: usize, gate: &Gate) { ... }
}
```

A type can also override a provided method. `QuantumState` overrides `probabilities` so it doesn't copy the amplitudes first.

## 4. `Vec<T>` and `vec!`

`Vec<T>` is Rust's growable array, stored contiguously on the heap. The state vector is a `Vec<Complex64>` of length `2^n`. The `vec![value; count]` macro builds a vector holding `count` copies of `value`, which is how the all-zero state is allocated in one step.

Indexing with `v[i]` is bounds-checked and panics if `i` is out of range. `v.len()` gives the length, and `v.push(x)` appends (used for the GPU's queue of pending gates).

```rust
let mut amplitudes = vec![Complex64::new(0.0, 0.0); 1 << num_qubits];
amplitudes[0] = Complex64::new(1.0, 0.0);   // set |00...0> = 1

let mut pending = Vec::new();
pending.push(params);
pending.len();
pending.clear();
```

## 5. Complex numbers (`num_complex::Complex64`)

Quantum amplitudes are complex numbers. The `num-complex` crate provides `Complex<T>`, and `Complex64` is the alias for `Complex<f64>`. It supports `+`, `-` and `*` directly through operator overloading, so a gate update reads like the maths: `g00 * x0 + g01 * x1`.

Methods used in the engine:

| Method | Meaning | Used for |
|---|---|---|
| `Complex64::new(re, im)` | build `re + i·im` (a `const fn`) | gate constants |
| `z.norm_sqr()` | `re² + im²` = \|z\|² | probabilities |
| `z.norm()` | \|z\| | test tolerances |
| `z.conj()` | complex conjugate | unitarity test |
| `z.re`, `z.im` | the two parts | GPU conversion |

```rust
use num_complex::Complex64;
let a = Complex64::new(0.0, 1.0);    // i
let p = a.norm_sqr();                // 1.0
let b = a * a;                       // -1 + 0i
```

## 6. Bitwise operators and bit masks

A basis state's index is a binary number with one bit per qubit, so bit operations are the core of the simulator. Rust uses the C-style operators. Most of the engine's indexing reduces to the patterns below.

| Operator | Name | Engine use |
|---|---|---|
| `1 << q` | shift left | `2^q`, the stride for qubit `q` |
| `a & b` | AND | test whether bits are set |
| `a \| b` | OR | set bits (`mask \|= bit`) |
| `a ^ b` | XOR | flip bits |
| `x - 1` on a power of two | – | mask of all lower bits |

```rust
let dim    = 1usize << num_qubits;          // 2^n
let stride = 1usize << target;              // distance between paired amplitudes
let bit_set = index & (1 << q) != 0;        // is qubit q |1> in this basis state?
if i0 & control_mask == control_mask { ... } // are ALL control qubits |1>?
```

**Control masks.** A mask is a number whose set bits mark the control qubits. CNOT with control 2 uses `mask = 0b100`. A basis state passes when `index & mask == mask`. With `mask = 0` every state passes, which gives an ordinary single-qubit gate, so one function covers both cases.

## 7. Ownership, borrowing, `&` and `&mut`

Every value in Rust has exactly one owner, and the value is freed when its owner goes out of scope. Other code can **borrow** it. Any number of shared borrows `&T` (read-only) are allowed, *or* exactly one mutable borrow `&mut T`, but never both at once. The compiler enforces this rule, and it's why the multithreaded code can't have data races.

That's why gates take `&Gate` (they only read the matrix) and backends take `&mut self` (they change the state). It's also why section 8's `split_at_mut` matters: it gives two `&mut` borrows that provably don't overlap.

```rust
fn apply_gate(&mut self, target: usize, gate: &Gate) -> Result<()>
//            ^^^^^^^^^ exclusive, may modify   ^^^^^ shared, read-only

let [[g00, g01], [g10, g11]] = *gate;   // `*` dereferences: copy the matrix out
```

## 8. Slices: `chunks_mut` and `split_at_mut`

A **slice** `&[T]` or `&mut [T]` is a view into part of a `Vec` or array, made of a pointer and a length. The gate kernel depends on two slice methods.

`chunks_mut(n)` splits a slice into consecutive non-overlapping `&mut` pieces of length `n`. For target qubit `t`, the state splits into blocks of `2 · 2^t`.

`split_at_mut(k)` cuts one `&mut` slice into two disjoint `&mut` slices `[0..k)` and `[k..)`. Within each block, the first half holds the `i0` amplitudes and the second half the `i1` amplitudes, so this gives safe simultaneous write access to both members of every pair, with no `unsafe` code.

```rust
for chunk in self.amplitudes.chunks_mut(block) {   // block = 2 * stride
    let (lo, hi) = chunk.split_at_mut(stride);     // lo[j] pairs with hi[j]
    for (a0, a1) in lo.iter_mut().zip(hi) { ... }
}
```

## 9. Iterators

An iterator produces a sequence of values on demand. Rust iterators are *lazy*: adapters like `map` describe work, and a consumer like `collect` or `for_each` runs it. They compile down to the same machine code as a hand-written loop.

| Function | What it does |
|---|---|
| `.iter()` | yields `&T` |
| `.iter_mut()` | yields `&mut T` |
| `a.zip(b)` | walks two iterators in lockstep, yielding `(a, b)` |
| `.enumerate()` | adds an index: yields `(i, item)` |
| `.map(f)` | transforms each item |
| `.collect()` | gathers into a collection (type inferred, e.g. `Vec<f64>`) |
| `.chunks_exact(2)` | non-overlapping pieces of exactly 2 |
| `.into_iter()` | consumes the collection, yielding owned items |

```rust
// probabilities = |amp|^2 for every amplitude
self.amplitudes().iter().map(|a| a.norm_sqr()).collect()

// walk two slices together, with index j
for (j, (a0, a1)) in lo.iter_mut().zip(hi).enumerate() { ... }
```

## 10. Closures

A closure is an anonymous function that can capture variables from the scope around it. The syntax is `|args| body` or `|args| { ... }`. The gate kernel defines its pair update once as a closure, `update`, and calls it from both the parallel-across-blocks path and the parallel-within-block path. The closure captures `g00..g11` and `control_mask` from the enclosing function.

Closures are also what you pass to iterator adapters (`map`, `for_each`) and to Rayon. Because a closure that only reads its captures is `Fn + Sync`, Rayon can call it from many threads at once.

```rust
let update = |i0: usize, a0: &mut Complex64, a1: &mut Complex64| {
    if i0 & control_mask == control_mask {
        let (x0, x1) = (*a0, *a1);
        *a0 = g00 * x0 + g01 * x1;
        *a1 = g10 * x0 + g11 * x1;
    }
};
update(base + j, a0, a1);
```

---

# Part 2: Errors and API design

## 11. `Result`, `Ok`, `Err` and the `?` operator

Rust has no exceptions. A function that can fail returns `Result<T, E>`: either `Ok(value)` or `Err(error)`. The caller must handle both, because the compiler warns about an unused `Result`. qsengine returns `Result` from every gate, so a bad qubit index becomes a clean error rather than a crash. That error will map directly to a Python `ValueError` later.

The `?` operator means "if this is `Err`, return it from the current function; otherwise unwrap the `Ok` value". It keeps error handling to one character per call. `.unwrap()` extracts the value and panics on `Err`; it's used only in tests. `.map_err(f)` converts one error type into another, and `.as_ref()` borrows the contents of a `Result`.

```rust
fn bell() -> qsengine::Result<()> {
    let mut s = best_backend(2)?;   // returns early on Err
    s.h(0)?;
    s.cx(0, 1)?;
    Ok(())
}

if q < n { Ok(()) } else { Err(QsError::QubitOutOfRange { qubit: q, num_qubits: n }) }
```

## 12. Enums and `match`

A Rust `enum` is a type that is *one of* several variants, and each variant can carry its own data. `QsError` lists every way the engine can fail. `match` branches on the variant and pulls out its data, and the compiler checks that every variant is handled.

`if let` is a one-branch `match`, used in `best_backend` to try the GPU. `matches!(value, Pattern)` returns a `bool` and is handy in tests.

```rust
pub enum QsError {
    TooManyQubits { requested: usize, max: usize },   // struct-like variant
    DuplicateQubit(usize),                            // tuple-like variant
    Gpu(String),
}

match self {
    QsError::DuplicateQubit(q) => write!(f, "qubit {q} used twice"),
    QsError::Gpu(msg)          => write!(f, "GPU error: {msg}"),
    ...
}

if let Ok(gpu) = GpuQuantumState::new(n) { return Ok(Box::new(gpu)); }
```

## 13. Implementing `Display` and `Error`

`std::fmt::Display` controls how a type prints with `{}`. Implementing it gives `QsError` readable messages. Implementing the marker trait `std::error::Error` (empty, since its methods have defaults) lets `QsError` work with the wider Rust error ecosystem, including PyO3's conversion to Python exceptions.

```rust
impl fmt::Display for QsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self { ... => write!(f, "...") }
    }
}
impl std::error::Error for QsError {}
```

## 14. Trait objects: `dyn Trait` and `Box<dyn Trait>`

Usually Rust knows the concrete type at compile time. Sometimes the choice is made at *runtime*: GPU if one is present, CPU otherwise. A **trait object** `dyn QuantumBackend` means "some type that implements `QuantumBackend`", and its method calls go through a table of function pointers (dynamic dispatch).

Because the concrete type's size isn't known, a trait object always sits behind a pointer: `&dyn T`, `&mut dyn T` or `Box<dyn T>` (an owned heap allocation). Adding `+ Send` requires the value to be safe to move between threads, which PyO3 needs. `&mut dyn RngCore` is used the same way so any random generator can be passed in.

```rust
pub fn best_backend(n: usize) -> Result<Box<dyn QuantumBackend + Send>> {
    if let Ok(gpu) = GpuQuantumState::new(n) { return Ok(Box::new(gpu)); }
    Ok(Box::new(QuantumState::new(n)?))
}

fn time_circuit(state: &mut dyn QuantumBackend, ...) { state.h(0)?; }
```

## 15. Modules, `pub` and `use`

Each `.rs` file is a module. `lib.rs` declares the others with `mod name;`, and `pub mod` makes a module visible outside the crate. Items are private by default; `pub` exposes them, and `pub(crate)` exposes them only inside this crate (used for the shared test suite). `use` brings names into scope, and `pub use` re-exports them so users can write `qsengine::QuantumState` instead of `qsengine::engine::QuantumState`.

`crate::` is an absolute path from the crate root, and `super::` refers to the parent module.

```rust
// lib.rs
pub mod backend;
pub mod engine;
pub use engine::QuantumState;

// engine.rs
use crate::backend::QuantumBackend;
use rayon::prelude::*;          // `*` imports everything in `prelude`
```

## 16. Type aliases

`type Name = ExistingType;` creates a shorter name for an existing type without creating a new type. qsengine uses aliases to keep signatures readable and to change a type in one place.

```rust
pub type Gate = [[Complex64; 2]; 2];                    // 2x2 matrix
pub type Result<T> = std::result::Result<T, QsError>;   // generic alias
type Make = fn(usize) -> Box<dyn QuantumBackend>;       // function-pointer type
```

`[[T; 2]; 2]` is a fixed-size array of arrays. It lives on the stack, needs no allocation and is `Copy`, which makes it ideal for small matrices.

## 17. `const` and `const fn`

`const` declares a value computed at compile time and inlined wherever it's used. The fixed gates (`X`, `H`, ...) and tuning numbers (`MIN_PARALLEL_LEN`, `MAX_QUBITS`) are consts. A `const fn` is a function the compiler can run at compile time, which is why the helper `c(re, im)` can build the gate constants.

Gates with a parameter, such as `rx(theta)`, can't be constants, so they're ordinary functions returning a `Gate`.

```rust
const fn c(re: f64, im: f64) -> Complex64 { Complex64::new(re, im) }
pub const X: Gate = [[O, I], [I, O]];
pub const MAX_QUBITS: usize = 30;
const MIN_PARALLEL_LEN: usize = 1 << 14;
```

---

# Part 3: Performance

## 18. Rayon: parallel iterators

Rayon turns ordinary iterators into multithreaded ones. Replace `iter_mut()` with `par_iter_mut()` (or `chunks_mut` with `par_chunks_mut`), and Rayon splits the work across a thread pool, balancing load with work-stealing. Because of the borrowing rules (section 7), code that compiles is free of data races.

`with_min_len(n)` stops Rayon from splitting work into pieces smaller than `n` items. Without it, tiny tasks cost more to schedule than to compute. `for_each` runs a closure on every item, and `collect` gathers results in order.

The engine picks between two strategies. When the target qubit is low, there are many small blocks, so it parallelises *across* blocks. When the target qubit is high, there are a few huge blocks, so it parallelises *within* each block.

```rust
use rayon::prelude::*;

self.amplitudes
    .par_chunks_mut(block)
    .enumerate()
    .with_min_len((MIN_PARALLEL_LEN / stride).max(1))
    .for_each(|(b, chunk)| { ... });

lo.par_iter_mut().zip(hi.par_iter_mut()).enumerate()
  .with_min_len(MIN_PARALLEL_LEN)
  .for_each(|(j, (a0, a1))| update(base + j, a0, a1));

self.amplitudes.par_iter().map(|a| a.norm_sqr()).collect()
```

## 19. Iterator extras: `scan`, `fold`, `filter`, `count`

- **`scan(init, f)`** carries running state through the iterator. The engine uses it to build the cumulative probability array (a prefix sum) for sampling.
- **`fold(init, f)`** reduces all items to one value. The tests use it to find the largest CPU/GPU difference.
- **`filter(pred)`** keeps items matching a condition, and **`count()`** counts them.

```rust
probs.iter().scan(0.0, |acc, p| { *acc += p; Some(*acc) }).collect()

a.iter().zip(&b).map(|(x, y)| (x - y).norm()).fold(0.0, f64::max)
```

## 20. `partition_point` (binary search)

`slice.partition_point(pred)` returns the first index where `pred` becomes false, assuming the slice is ordered so that `pred` is true for a prefix and then false. It runs in `O(log n)`. On the cumulative probability array, it finds which basis state a random number lands in, which makes each measurement shot `O(n)` in the qubit count instead of `O(2^n)`.

```rust
let r = rng.random::<f64>() * total;
let idx = cumulative.partition_point(|&c| c <= r);
```

The `|&c|` pattern destructures the `&f64` reference so `c` is a plain `f64`.

## 21. `fill` and `clone`

`slice.fill(value)` overwrites every element and compiles to a fast memory set. Measurement collapse uses it to zero the state. `.clone()` makes a deep copy, used when `amplitudes()` must return an owned `Vec` without giving away the state.

```rust
self.amplitudes.fill(Complex64::new(0.0, 0.0));
self.amplitudes.clone()
```

## 22. Integer helpers

| Function | Result | Engine use |
|---|---|---|
| `a.div_ceil(b)` | ⌈a / b⌉ | GPU workgroup count |
| `a.next_multiple_of(b)` | round up to a multiple of b | uniform-buffer alignment |
| `x.leading_zeros()` | number of leading 0 bits | `floor(log2(x)) = 63 - leading_zeros` |
| `a.min(b)` / `a.max(b)` | smaller / larger | clamping |
| `x as u32` | numeric cast | `usize` → GPU `u32` |
| `size_of::<T>()` | bytes in type `T` | buffer sizes |

```rust
let groups = (pairs).div_ceil(256).max(1) as u32;
let align  = (size_of::<GateParams>() as u64).next_multiple_of(min_align);
```

## 23. `f64` maths

`theta.sin_cos()` returns `(sin, cos)` in one call, which is faster than calling each separately. It's used by the rotation gates. `std::f64::consts` provides exact constants such as `PI` and `FRAC_1_SQRT_2` (1/√2, the Hadamard coefficient).

```rust
let (s, co) = (theta / 2.0).sin_cos();
use std::f64::consts::FRAC_1_SQRT_2;
```

---

# Part 4: Randomness and collections

## 24. `rand`

The `rand` crate (version 0.9) supplies random numbers for measurement.

- `rand::rng()` gives a fast, automatically seeded generator for normal use.
- `rng.random::<f64>()` returns a uniform number in `[0, 1)`.
- `RngCore` is the base trait of every generator. Measurement takes `&mut dyn RngCore` so callers can pass any generator, including a seeded one.
- `StdRng::seed_from_u64(seed)` (from `SeedableRng`) makes a reproducible generator, used in tests.

```rust
use rand::{Rng, RngCore, SeedableRng, rngs::StdRng};

let mut rng = rand::rng();
let m = state.measure(&mut rng);

let mut seeded = StdRng::seed_from_u64(42);    // same results every run
```

## 25. `HashMap` and the `entry` API

`HashMap<K, V>` is a key-value map. `sample()` returns counts as `HashMap<usize, usize>` (basis index → times seen), which becomes a Python `dict` naturally. `entry(key).or_insert(0)` returns a mutable reference to the value, inserting `0` first if the key is missing, so counting takes one line.

```rust
let mut counts = HashMap::new();
*counts.entry(index).or_insert(0) += 1;
counts[&0]                // read (panics if missing)
counts.keys()
```

---

# Part 5: GPU and concurrency

## 26. `OnceLock`

`std::sync::OnceLock<T>` holds a value that's initialised exactly once, even when many threads race to do it. Opening a GPU device and compiling the shader take about 100 ms, so qsengine does it once per process in a `static OnceLock`, and every `GpuQuantumState` borrows the result. The failure case (no GPU) is cached too, so later calls fail instantly.

```rust
static CONTEXT: OnceLock<Result<GpuContext, String>> = OnceLock::new();
CONTEXT.get_or_init(|| pollster::block_on(GpuContext::create()))
```

A `&'static GpuContext` is a reference that lives for the whole program, which is exactly what a `static` provides.

## 27. `Mutex` and `get_mut`

`Mutex<T>` allows mutation through a shared reference by locking. The GPU backend keeps its pending-gate queue in a `Mutex` because `amplitudes(&self)` must flush the queue, which is a write, through a read-only `&self`. A `RefCell` would also work but isn't thread-safe; a `Mutex` keeps the type `Send + Sync`, which PyO3 classes require.

`lock()` blocks until the lock is free and returns a guard that unlocks when dropped. When you already hold `&mut self`, `get_mut()` gives access with no locking cost, because the compiler has already proven exclusive access.

```rust
pending: Mutex<Vec<GateParams>>,

let mut q = self.pending.lock().unwrap();       // from &self
let q = self.pending.get_mut().unwrap();        // from &mut self, free
```

## 28. `async`, `.await` and `pollster::block_on`

wgpu's setup functions are `async`: they return a *future*, a value that completes later. Inside an `async fn`, `.await` waits for a future. A plain synchronous program needs something to drive the future to completion, and `pollster::block_on(future)` is the smallest such runtime: it blocks the current thread until the future finishes.

```rust
async fn create() -> Result<Self, String> {
    let adapter = instance.request_adapter(&opts).await.map_err(...)?;
    ...
}
pollster::block_on(GpuContext::create())
```

## 29. `#[repr(C)]` and `bytemuck`

By default Rust can reorder struct fields. `#[repr(C)]` fixes the field order and padding so they match what the GPU shader expects byte-for-byte. The `bytemuck` crate then reinterprets data as raw bytes, or bytes as typed data, without copying and without `unsafe`. Deriving `Pod` (plain old data) and `Zeroable` proves to the compiler that this is safe.

```rust
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GateParams { matrix: [f32; 8], target_qubit: u32, control_mask: u32, num_pairs: u32, _pad: u32 }

bytemuck::bytes_of(&params)                 // &GateParams -> &[u8]
bytemuck::cast_slice::<u8, f32>(&data)      // &[u8] -> &[f32]
bytemuck::cast_slice(&[1.0f32, 0.0])        // &[f32] -> &[u8]
```

## 30. The wgpu functions

wgpu is a Rust implementation of the WebGPU standard. It runs on Vulkan, DirectX 12 and Metal, so it works on NVIDIA, AMD, Intel and Apple GPUs. The engine uses these calls, in lifecycle order:

| Call | Purpose |
|---|---|
| `Instance::new(&desc)` | entry point into wgpu |
| `instance.request_adapter(&opts)` | find a physical GPU (`HighPerformance` prefers a discrete one) |
| `adapter.get_info().device_type` | detect software-only "GPUs" and skip them |
| `adapter.limits()` | the hardware's real maximum buffer sizes etc. |
| `adapter.request_device(&desc)` | open a logical device + command queue |
| `include_wgsl!("file")` | embed the shader source at compile time |
| `device.create_shader_module` | compile the shader |
| `device.create_bind_group_layout` | declare what buffers the shader takes |
| `device.create_pipeline_layout` / `create_compute_pipeline` | build the runnable program |
| `device.create_buffer(&desc)` | allocate GPU memory (zero-initialised) |
| `device.create_bind_group` | attach real buffers to the layout |
| `queue.write_buffer(buf, offset, bytes)` | upload data |
| `device.create_command_encoder` | start recording GPU commands |
| `encoder.begin_compute_pass` | start a compute section |
| `pass.set_pipeline` / `set_bind_group(0, bg, &[offset])` | choose the program and inputs; the dynamic offset picks which gate's parameters |
| `pass.dispatch_workgroups(x, y, 1)` | launch `x*y` groups of 256 threads |
| `encoder.copy_buffer_to_buffer` | GPU-side copy (for reading back) |
| `encoder.clear_buffer` | GPU-side zero fill (measurement collapse) |
| `queue.submit([encoder.finish()])` | send the recorded commands to run |
| `slice.map_async(MapMode::Read, cb)` + `device.poll(PollType::Wait)` | wait for results and make them CPU-readable |
| `slice.get_mapped_range()` | the bytes, as a slice |

**Buffer usages** (`STORAGE`, `UNIFORM`, `COPY_SRC`, `COPY_DST`, `MAP_READ`) are bit flags combined with `|` that declare what each buffer is allowed to do.

**Batching.** Every gate call only *records* its parameters. On the next read, all gates are packed into one uniform buffer and run in a single submission with one dispatch per gate, which avoids a slow CPU↔GPU round trip per gate.

## 31. WGSL

WGSL is WebGPU's shader language, and `apply_gate.wgsl` is the GPU version of the CPU kernel. It looks like Rust: `let` bindings, `fn`, `struct`, `vec2<f32>`, `u32`. `@compute @workgroup_size(256)` marks the entry point, and each invocation gets its thread number from `@builtin(global_invocation_id)`. `var<storage, read_write>` is the state vector, and `var<uniform>` is the small read-only parameter block.

WGSL has no portable 64-bit floats, so GPU amplitudes are `f32`. Results match the CPU to about 1e-6, which is fine for teaching and most simulation. Use `QuantumState` when you need full `f64` precision.

```wgsl
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>, ...) {
    let k = gid.x + gid.y * nwg.x * 256u;   // flatten a 2D dispatch
    if (k >= p.num_pairs) { return; }
    let i0 = ((k - low) << 1u) | low;       // insert a 0 bit at the target
    ...
}
```

---

# Part 6: Attributes, macros and tooling

## 32. `#[derive(...)]`

`#[derive]` asks the compiler to write standard trait implementations automatically.

- `Debug` enables printing with `{:?}`.
- `Clone` adds `.clone()`.
- `Copy` makes assignment copy bits instead of moving.
- `PartialEq` adds `==`, which tests use to compare errors.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum QsError { ... }
```

## 33. Cargo features and `#[cfg(...)]`

Cargo **features** are optional, named parts of a crate. qsengine's `gpu` feature pulls in wgpu, pollster and bytemuck. It's on by default, and `cargo build --no-default-features` produces a CPU-only build for platforms where wgpu isn't wanted. In code, `#[cfg(feature = "gpu")]` compiles the next item only when the feature is on, and `#[cfg(test)]` compiles it only for `cargo test`.

```toml
[features]
default = ["gpu"]
gpu = ["dep:wgpu", "dep:pollster", "dep:bytemuck"]
```
```rust
#[cfg(feature = "gpu")]
pub mod gpu;
```

The GPU is also optional *at runtime*. `best_backend` tries it and silently falls back to the CPU, so one program runs on every machine.

## 34. `format!`, `println!`, `write!`

These macros format text. `{}` uses `Display`, `{:?}` uses `Debug`, and variables can be named inline as `{name}`. Format specs follow a colon: `{:>8.1}` right-aligns to width 8 with 1 decimal place, and `{index:0num_qubits$b}` prints binary, zero-padded to a width taken from the variable `num_qubits`. `write!` writes into a formatter (inside `Display`), and `eprintln!` prints to stderr.

```rust
format!("{index:0num_qubits$b}")         // 6, 3  -> "110"
println!("  |{}>  {n}", format_bitstring(index, 2));
println!("{cpu_time:>8.1} ms");
```

## 35. Tests

A function marked `#[test]` runs under `cargo test`. Tests normally live in a `#[cfg(test)] mod tests` at the bottom of each file. `use super::*;` imports the file's items.

- `assert!(cond, "msg {x}")` fails if `cond` is false.
- `assert_eq!(a, b)` fails if they differ and prints both.
- `matches!(v, Pattern)` checks a variant.

qsengine has a shared **conformance suite** (`backend::conformance::run_all`), and every backend must pass it. Code in `///` comments that's marked as a code block also runs as a *doc test*.

```rust
#[test]
fn passes_backend_conformance_suite() {
    conformance::run_all(|n| Box::new(QuantumState::new(n).unwrap()), 1e-12);
}
```

```bash
cargo test --release                       # everything (GPU tests skip without a GPU)
cargo test --release --no-default-features # CPU-only build
```

## 36. Doc comments

`///` documents the item that follows, and `//!` documents the enclosing module or crate. Both support Markdown and are turned into an HTML site by `cargo doc --open`. Ordinary `//` comments aren't included.

```rust
//! CPU backend: a multithreaded state vector.

/// Create the all-zero state |0...0>.
pub fn new(num_qubits: usize) -> Result<Self>
```

## 37. `Instant` and `black_box`

`std::time::Instant::now()` captures a monotonic timestamp, and `.elapsed()` returns a `Duration` (`.as_secs_f64()` converts it to seconds). `std::hint::black_box(x)` tells the optimiser that `x` is used, so it can't delete benchmarked work whose result would otherwise be ignored.

```rust
let start = Instant::now();
/* ... run circuit ... */
std::hint::black_box(state.probabilities());
let ms = start.elapsed().as_secs_f64() * 1000.0;
```


---

# Part 7: Python bindings (PyO3)

PyO3 is the bridge between Rust and Python. Its attributes generate the C-level glue that CPython expects, so a Rust struct shows up as an ordinary Python class. All of qsengine's bindings live in `src/python.rs`, which only translates arguments and results. The simulation itself stays in `circuit.rs` and the backends.

## 38. `#[pyclass]` and `#[pymethods]`

`#[pyclass]` turns a Rust struct into a Python class, and `name = "..."` sets the name Python sees. The struct keeps its fields private to Python: qsengine's `PyQuantumCircuit` just wraps the pure-Rust `QuantumCircuit`. That way the engine never depends on PyO3, and Rust users never pay for it.

`#[pymethods]` on an `impl` block exposes every method in it to Python. `&self` methods are read-only calls and `&mut self` methods may modify the object; PyO3 enforces Rust's borrow rules at runtime. A pyclass must be `Send`, which is why the GPU backend uses a `Mutex` (section 27).

```rust
#[pyclass(name = "QuantumCircuit", module = "qsengine")]
struct PyQuantumCircuit { inner: QuantumCircuit }

#[pymethods]
impl PyQuantumCircuit {
    /// Hadamard on `qubit`.            <- becomes the Python docstring
    fn h(&mut self, qubit: usize) -> PyResult<()> { Ok(self.inner.h(qubit)?) }
}
```

Arguments convert automatically: a Python `int` becomes `usize` (a negative number raises `OverflowError`), a `float` becomes `f64` and a `str` becomes `&str`. Return values convert back: `Vec<f64>` → `list[float]`, `HashMap<String, usize>` → `dict[str, int]`, and `Complex64` → `complex` (thanks to PyO3's `num-complex` feature).

## 39. `#[new]`, `#[getter]` and dunder methods

`#[new]` marks the constructor, which Python calls as `QuantumCircuit(3)`. `#[getter]` exposes a method as a read-only property, so `qc.num_qubits` takes no parentheses. Methods named like Python's special methods are wired up automatically: `__len__` powers `len(qc)` and `__repr__` controls how the object prints.

```rust
#[new]
fn new(num_qubits: usize) -> PyResult<Self> { ... }

#[getter]
fn num_qubits(&self) -> usize { ... }

fn __len__(&self) -> usize { ... }
fn __repr__(&self) -> String { ... }
```

## 40. `PyResult` and converting errors with `From`

`PyResult<T>` is `Result<T, PyErr>`. Returning `Err` raises a Python exception. qsengine implements `From<QsError> for PyErr`, so the `?` operator converts engine errors automatically: input mistakes become `ValueError` and GPU failures become `RuntimeError`. The `From` trait is Rust's standard conversion hook, and `?` calls it whenever the error types differ.

`PyValueError::new_err(msg)` builds an exception directly, which is how an unknown `backend=` name is reported.

```rust
impl From<QsError> for PyErr {
    fn from(e: QsError) -> PyErr {
        match e {
            QsError::Gpu(_) => PyRuntimeError::new_err(e.to_string()),
            _ => PyValueError::new_err(e.to_string()),
        }
    }
}

fn h(&mut self, qubit: usize) -> PyResult<()> { Ok(self.inner.h(qubit)?) }
//                                                     `?` converts ^
```

## 41. `#[pyo3(signature = ...)]` and `Option<T>`

Rust has no default arguments, but Python users expect them. `#[pyo3(signature = (...))]` declares each parameter's default value as Python sees it. `Option<T>` is Rust's "maybe a value" type: `Some(x)` or `None`. In a binding, an `Option` parameter accepts Python's `None`, so `seed: Option<u64>` means "an int, or None".

`match` on an `Option` picks the behaviour: a seeded generator for reproducible runs, or OS randomness.

```rust
#[pyo3(signature = (shots = 1024, seed = None, backend = "auto"))]
fn run(&self, py: Python<'_>, shots: usize, seed: Option<u64>, backend: &str) -> PyResult<...>

let mut rng = match seed {
    Some(s) => StdRng::seed_from_u64(s),
    None => StdRng::from_os_rng(),
};
```

## 42. `py.detach`: releasing the GIL

CPython's Global Interpreter Lock (GIL) lets only one thread run Python code at a time. Holding it during a long simulation would freeze every other Python thread. `py.detach(closure)` releases the GIL, runs the closure (which must not touch Python objects) and takes the GIL back. Taking a `py: Python<'_>` parameter gives a method this token.

The closure uses `move` so that it owns its data. The circuit is cloned first, because `self` belongs to Python and can't be sent across while detached. Cloning a list of small instructions is cheap compared with simulating `2^n` amplitudes.

```rust
let circuit = self.inner.clone();
let counts = py.detach(move || circuit.run(shots, kind, &mut rng))?;
```

## 43. `#[pymodule]` and `Bound`

`#[pymodule]` marks the function CPython calls on `import qsengine`. Its name must match the library name in `Cargo.toml`. It receives the new module as `&Bound<'_, PyModule>`. A `Bound<'py, T>` is a reference to a Python object that's valid while the GIL is held, and the `'py` lifetime is how PyO3 makes that a compile-time rule. `m.add_class` registers a class, and `m.add` sets a module attribute.

```rust
#[pymodule]
fn qsengine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyQuantumCircuit>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;   // env! reads a build-time variable
    Ok(())
}
```

## 44. maturin, `cdylib` and abi3

A Python extension is a shared library (`.pyd` on Windows, `.so` elsewhere). `crate-type = ["cdylib", "rlib"]` makes Cargo build both the shared library and the normal Rust library that the demo and the tests use. The bindings sit behind the `python` Cargo feature, so plain `cargo build` and `cargo test` don't need Python at all. maturin switches the feature on through `pyproject.toml`.

PyO3's `abi3-py39` feature compiles against CPython's *stable ABI*, so one wheel works on Python 3.9, 3.10, ... 3.14 and later. That makes distribution much easier: one wheel per OS and CPU instead of one per Python version. `qsengine.pyi` is a type stub that maturin ships alongside the extension, giving IDEs autocomplete for a module that has no `.py` source.

```bash
maturin develop --release     # build and install into the active venv (for development)
maturin build --release       # produce a wheel in target/wheels/ for distribution
```

---

## Quick reference: commands

```bash
cargo run --release                        # Bell-state demo + CPU vs GPU benchmark
cargo test --release                       # all tests
cargo build --release --no-default-features # CPU-only (no wgpu)
cargo doc --open                           # browse the API docs
maturin develop --release                  # build + install the Python module into the venv
pytest                                     # Python tests (against NumPy)
```
