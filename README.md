# Array Buffer

[![Crate](https://img.shields.io/crates/v/array_buf.svg)](https://crates.io/crates/array_buf)
[![API](https://docs.rs/array_buf/badge.svg)](https://docs.rs/array_buf)

Highly optimized fixed-capacity deque buffer stored on the stack.

### Todo:

- `iter()` is not implemented, but there is `as_slices()`;
- Can become a true ring buffer. It can overwrite an old element, but it needs to drop it (for not plain). Increment both `start` and `end` (`start == end`);
