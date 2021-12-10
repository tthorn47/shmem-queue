use crate::{shmem, QUEUE_SIZE};
use core::cell::Cell;
use core::mem::size_of;
use core::sync::atomic::{AtomicUsize, Ordering};

type QueueEntry<T> = [Cell<T>; QUEUE_SIZE];

pub struct Queue<'a, T> {
    log: &'a QueueEntry<T>,
    head: *const AtomicUsize,
    tail: *const AtomicUsize,
}

/// This is just to make the compiler happy
/// when using queue with multiple threads.
unsafe impl<'a, T> Send for Queue<'a, T> {}
unsafe impl<'a, T> Sync for Queue<'a, T> {}

impl<'a, T> Queue<'a, T>
where
    T: Sized + Default + Copy + Clone,
{
    pub fn new(name: &str) -> Queue<'a, T> {
        let buffer_size = size_of::<QueueEntry<T>>() + size_of::<AtomicUsize>() * 2;
        let inner = shmem::create_shm(name, buffer_size);

        let log = unsafe { &mut *(inner as *mut QueueEntry<T>) };
        for e in log.iter_mut() {
            *e = Default::default();
        }
        let head = unsafe { inner.add(size_of::<QueueEntry<T>>()) } as *mut AtomicUsize;
        let tail = unsafe { inner.add(size_of::<QueueEntry<T>>() + size_of::<AtomicUsize>()) }
            as *mut AtomicUsize;
        Queue { log, head, tail }
    }

    fn head(&self) -> usize {
        unsafe { (*self.head).load(Ordering::Acquire) }
    }

    fn tail(&self) -> usize {
        unsafe { (*self.tail).load(Ordering::Acquire) }
    }

    pub fn enqueue(&self, value: T) -> bool {
        let head = self.head();
        let tail = self.tail();
        let next = (head + 1) % QUEUE_SIZE;
        if next == tail {
            return false;
        }
        unsafe {
            (*self.log.get_unchecked(head)).set(value);
            (*self.head).store(next, Ordering::Release);
        }
        true
    }

    pub fn dequeue(&self) -> Option<T> {
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
    fn test_dequeue() {
        let queue = Queue::<i32>::new("test");
        assert!(queue.enqueue(1));
        assert_eq!(queue.head(), 1);
        assert_eq!(queue.tail(), 0);

        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.head(), 1);
        assert_eq!(queue.tail(), 1);
    }

    #[test]
    fn test_equeue_full() {
        let queue = Queue::<i32>::new("test");
        for i in 0..QUEUE_SIZE - 1 {
            assert!(queue.enqueue(i as i32));
        }
        assert!(queue.tail() == 0);
        assert!(queue.head() == QUEUE_SIZE - 1);
        assert!(!queue.enqueue(QUEUE_SIZE as i32));
    }

    #[test]
    fn test_dequeue_empty() {
        let queue = Queue::<i32>::new("test");
        assert_eq!(queue.dequeue(), None);
    }

    #[test]
    fn test_two_clients() {
        let producer = Queue::<i32>::new("test");
        let consumer = Queue::<i32>::new("test");

        assert!(producer.enqueue(1));
        assert_eq!(producer.head(), 1);
        assert_eq!(producer.tail(), 0);

        assert_eq!(consumer.dequeue(), Some(1));
        assert_eq!(consumer.head(), 1);
        assert_eq!(consumer.tail(), 1);
    }

    #[test]
    fn test_parallel_client() {
        let producer = Queue::<i32>::new("test");
        let consumer = Queue::<i32>::new("test");
        let num_iterations = 10 * QUEUE_SIZE;

        let producer_thread = std::thread::spawn(move || {
            for i in 0..num_iterations {
                loop {
                    if producer.enqueue(i as i32) {
                        break;
                    }
                }
            }
        });

        let consumer_thread = std::thread::spawn(move || {
            for i in 0..num_iterations {
                loop {
                    if let Some(value) = consumer.dequeue() {
                        assert_eq!(value, i as i32);
                        break;
                    }
                }
            }
        });

        producer_thread.join().unwrap();
        consumer_thread.join().unwrap();
    }
}
