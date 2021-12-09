#![no_std]

#[cfg(test)]
extern crate std;

extern crate alloc;
extern crate libc;

const QUEUE_SIZE: usize = 1024;

mod queue;
mod shmem;

use queue::Queue;

#[repr(transparent)]
pub struct Sender<'a, T>(Queue<'a, T>);

impl<'a, T> Sender<'a, T>
where
    T: Sized + Default + Copy + Clone,
{
    pub fn new(name: &str) -> Sender<'a, T> {
        Sender(Queue::<T>::new(name))
    }

    pub fn send(&self, data: T) -> bool {
        loop {
            if self.0.enqueue(data) {
                return true;
            }
        }
    }

    pub fn try_send(&self, data: T) -> bool {
        self.0.enqueue(data)
    }
}

#[repr(transparent)]
pub struct Receiver<'a, T>(Queue<'a, T>);

impl<'a, T> Receiver<'a, T>
where
    T: Sized + Default + Copy + Clone,
{
    pub fn new(name: &str) -> Receiver<'a, T> {
        Receiver(Queue::<T>::new(name))
    }

    pub fn recv(&self) -> T {
        loop {
            if let Some(data) = self.0.dequeue() {
                return data;
            }
        }
    }

    pub fn try_recv(&self) -> Option<T> {
        self.0.dequeue()
    }
}
