# `CowVec`

One cool feature of `std::borrow::Cow` is that if you do `Cow` of an array, it can act as either `&[T]` or `Vec<T>` (instead of, for example, `&T` or `T`). This is super useful, and especially when working with strings you may find that `Cow<'static, str>` is often the best type to default to for owned strings, since you can either take an owned, dynamically generated string or a string literal with zero cost. Of course, it does complicate your implementation and API so only use it when you're sure that it's actually a performance win - as always, your benchmarks should always have the final say.

I say "zero-cost", but there actually is a cost to call `Cow::as_ref`, since it has to check the tag first. This isn't necessary: the representation of `Vec` and `&T` are something like so:

```rust
struct Vec<T> {
    capacity: usize,
    len: usize,
    pointer: *mut T,
}

struct FatPointer<T> {
    len: usize,
    pointer: *const T,
}
```

Since `*mut` and `*const` are basically the same (the only difference is [variance][variance]), we could store a `Cow<[T]>` like so:

```rust
struct CowVec {
    capacity: usize,
    len: usize,
    pointer: *mut T,
}
```

Using `capacity == 0` to indicate that it's a slice. Since vectors with 0 capacity also have 0 length and don't have anything allocated on the heap, this is safe. This crate implements that optimisation, although I haven't actually benchmarked it against `std::borrow::Cow` yet and it hasn't been looked over for safety issues so your mileage may vary.

There's not really a way for a generic `Cow` to beat this, since there are too many moving parts for an optimiser to figure out that this is possible on its own. Rust doesn't allow userland code to change enum layouts, so the only way the generic `Cow` could implement this optimisation is by hard-coding it into the compiler.

One unanswered question is whether I should use `*mut T` for invariance, since `*mut T` is stricter than `*const T` and `Vec` uses `*mut`. This implies that `*mut` is the safer option.
