use shmem_queue::Receiver;

#[derive(Debug, Default, Clone, Copy)]
struct Message {
    secret: usize,
    random_number: usize,
}

fn main() {
    let iter = 10 * 1024;

    let receiver = Receiver::<Message>::new("queue");
    for i in 0..iter {
        let message = receiver.recv();
        assert_eq!(message.secret, 0xDEADBEEF);
        assert_eq!(message.random_number, i);
    }
}
