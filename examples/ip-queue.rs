use libc::pid_t;
use nix::sys::wait::wait;
use nix::unistd::ForkResult::{Child, Parent};
use nix::unistd::{fork, getpid, getppid};

use shmem_queue::{Receiver, Sender};

#[derive(Debug, Default, Clone, Copy)]
struct Message {
    pid: pid_t,
    random_number: u32,
}

fn main() {
    let iter = 10;
    let pid = unsafe { fork() };

    match pid.expect("Fork Failed: Unable to create child process!") {
        Child => {
            let receiver = Receiver::<Message>::new("queue");
            for i in 0..iter {
                let message = receiver.recv();
                assert_eq!(message.pid, getppid().as_raw());
                assert_eq!(message.random_number, i);
            }
        }
        Parent { child: _ } => {
            let sender = Sender::<Message>::new("queue");
            let mut message = Message {
                pid: getpid().as_raw(),
                random_number: 0,
            };
            for i in 0..iter {
                message.random_number = i;
                sender.send(message.clone());
            }

            let _ignore = wait();
        }
    }
}
