//! Deterministic zero-allocation polyphonic DSP engine.
#![cfg_attr(not(feature = "std"), no_std)]

pub mod envelope;
pub mod filter;
pub mod fx;
pub mod math;
pub mod osc;
