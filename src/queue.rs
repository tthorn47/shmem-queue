use crate::{shmem, QUEUE_SIZE};
use core::cell::Cell;
use core::mem::size_of;
use core::sync::atomic::{AtomicUsize, Ordering};

struct Queue<'a, T> {
    log: &'a [Cell<T>; QUEUE_SIZE],
    head: *const AtomicUsize,
    tail: *const AtomicUsize,
}

impl<'a, T> Queue<'a, T>
where
    T: Sized + Default + Copy + Clone,
{
    fn new(name: &str) -> Queue<'a, T> {
        let buffer_size = size_of::<[Cell<T>; QUEUE_SIZE]>() + size_of::<AtomicUsize>() * 2;
        let inner = shmem::create_shm(name, buffer_size);

        let log = unsafe { &mut *(inner as *mut [Cell<T>; QUEUE_SIZE]) };
        for e in log.iter_mut() {
            *e = Default::default();
        }
        let head = unsafe { inner.offset(size_of::<[Cell<T>; QUEUE_SIZE]>() as isize) }
            as *mut AtomicUsize;
        let tail = unsafe {
            inner.offset((size_of::<[Cell<T>; QUEUE_SIZE]>() + size_of::<AtomicUsize>()) as isize)
        } as *mut AtomicUsize;
        Queue { log, head, tail }
    }

    fn head(&self) -> usize {
        unsafe { (*self.head).load(Ordering::Acquire) }
    }

    fn tail(&self) -> usize {
        unsafe { (*self.tail).load(Ordering::Acquire) }
    }

    fn enqueue(&self, value: T) -> bool {
        let head = self.head();
        let tail = self.tail();

        if head - tail >= QUEUE_SIZE {
            return false;
        }

        let next_head = (head + 1) % QUEUE_SIZE;
        unsafe {
            *self.log[head as usize].as_ptr() = value;
            (&*self.head).store(next_head, Ordering::Release);
        }

        true
    }

    fn deqeue(&self) -> Option<T> {
        let head = self.head();
        let tail = self.tail();

        if head == tail {
            return None;
        }

        let next_tail = (tail + 1) % QUEUE_SIZE;
        unsafe {
            let value = *self.log[tail as usize].as_ptr().as_ref().unwrap();
            (&*self.tail).store(next_tail, Ordering::Release);
            Some(value)
        }
    }
}

#[cfg(test)]
impl<'a, T> Drop for Queue<'a, T> {
    fn drop(&mut self) {
        shmem::destroy_shm("test");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_initialization() {
        let queue = Queue::<i32>::new("test");
        assert!(queue.log.len() == QUEUE_SIZE);
        assert_eq!(queue.head(), 0);
        assert_eq!(queue.tail(), 0);
        for i in 0..QUEUE_SIZE {
            let ele = unsafe { queue.log[i].as_ptr().as_ref().unwrap() };
            assert!(*ele == 0);
        }
    }

    #[test]
    fn test_enqueue() {
        let queue = Queue::<i32>::new("test");
        assert!(queue.enqueue(1));
        assert_eq!(queue.head(), 1);
    }

    #[test]
    fn dequeue() {
        let queue = Queue::<i32>::new("test");
        assert!(queue.enqueue(1));
        assert_eq!(queue.head(), 1);
        assert_eq!(queue.tail(), 0);

        assert_eq!(queue.deqeue(), Some(1));
        assert_eq!(queue.head(), 1);
        assert_eq!(queue.tail(), 1);
    }
}
