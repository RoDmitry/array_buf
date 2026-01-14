//! # Array Buffer
//! 
//! Highly optimized fixed-capacity deque buffer stored on the stack.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

mod deque;

pub use deque::*;
