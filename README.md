# Asphalt

A (WIP) highly experimental async ORM for Rust (NOT production ready).

## Unstable features

The crate will use many unstable features, making it nightly-only. We wan't to both test
these features in real code and try to make the nicest interface the future of rust can
provide us.

This doesn't mean that we will use any feature added by the compiler. We strive to only
use features that have a clear path to stabilization _and_ aren't too much incomplete.

We track the currently used unstable features in [this issue](https://github.com/luisholanda/asphalt-orm/issues/3).

## Comparisson with [Diesel](https://github.com/diesel-rs/diesel)

The library is highly inspired by the work done in diesel, but have a few key differences:

* Instead of a monolithic crates, we have many crates, each one building on top of the others.
* A thin type-safe layer on top of untyped-layer (except by the backend type). This is done
to reduce the amount of generated code.
* We don't fear nightly: this crate was created as experiment on how we can improve the ORM
interface using the newest Rust features.

## How the crates are organized

The asphalt set of libraries is written in many different building blocks, which can be used
together as necessary. The following crates are already implemented:

* `asphalt_core`: The core types behind database communication, can be seen as an abstraction
between the user binary and the database itself.


## License

Licensed under the Apache License, Version 2.0 ([LICENSE](https://github.com/luisholanda/asphalt-orm/blobs/master/LICENSE) or [apache.org/licenses/LICENSE-2.0](https://apache.org/licenses/LICENSE-2.0)).
