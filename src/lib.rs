#![no_std]

#[cfg(test)]
extern crate std;

extern crate alloc;
extern crate libc;

const QUEUE_SIZE: usize = 1024;

mod queue;
mod shmem;

pub use queue::Queue;
