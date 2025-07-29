//! Efficiently stores a sequence of elements with very small representations.
//!
//! This uses enumeration in the math sense. Every unique sequence is mapped to a natural number.
//! However, we are still bounded by the number of representable numbers.
//!
//! This crate is named terribly, and so are its members.

#![no_std]

pub mod digit;

mod digit_deque;

pub mod element_deque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceFull;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceEmpty;
