//! # Array Buffers
//!
//! Highly optimized fixed-capacity buffers stored on the stack.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

mod bytes_arr;
mod deque;

pub use bytes_arr::*;
pub use deque::*;
