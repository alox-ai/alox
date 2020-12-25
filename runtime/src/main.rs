use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

use concache::crossbeam::{Map, MapHandle};
use crossbeam::queue::SegQueue;
use libc::c_void;
use rayon::{ThreadPool, ThreadPoolBuilder};

type ActorId = u64;
type ActorPtr = *mut c_void;

struct ActorManager {
    id_counter: AtomicU64,
    map: Map<ActorId, ActorPtr>,
}

impl ActorManager {
    fn new() -> ActorManager {
        ActorManager {
            id_counter: AtomicU64::new(0),
            map: Map::with_capacity(20),
        }
    }

    fn generate_id(&self) -> u64 {
        self.id_counter.fetch_add(1, Ordering::Relaxed)
    }

    fn create_actor(&self, size: usize) -> ActorPtr {
        let actor_ptr: ActorPtr = unsafe { libc::malloc(size as libc::size_t) };
        let id = self.generate_id();
        self.map.insert(id, actor_ptr);
        actor_ptr
    }
}

struct Message {
    actor_id: ActorId,
    method: String,
    args: Vec<u8>,
}

struct Scheduler {
    queue: SegQueue<Message>,
    worker_thread_pool: ThreadPool,
}

impl<'a> Scheduler {
    fn new(worker_thread_count: usize) -> Scheduler {
        Scheduler {
            queue: SegQueue::new(),
            worker_thread_pool: ThreadPoolBuilder::new().num_threads(worker_thread_count).build().unwrap(),
        }
    }

    fn add_message(&self, message: Message) {
        self.queue.push(message)
    }

    fn take_message(&self) -> Option<Message> {
        self.queue.pop()
    }
}

fn main() {
    println!("Hello, world!");
}
