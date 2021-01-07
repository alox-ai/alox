#[macro_use]
extern crate lazy_static;

use std::ffi::CString;
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicU64, Ordering};

use concache::crossbeam::Map;
use crossbeam::queue::SegQueue;
use libc::c_void;
use rayon::{ThreadPool, ThreadPoolBuilder};

type ActorId = u64;

// fn(this actor, referenced actors, behavior args)
type ExternalBehavior = unsafe extern "C" fn(*mut c_void, Vec<ActorId>, Vec<u8>);

#[derive(Clone, Copy, Debug)]
struct ActorPtr(*mut c_void);

unsafe impl Send for ActorPtr {}
unsafe impl Sync for ActorPtr {}

struct ActorManager {
    id_counter: AtomicU64,
    map: Map<ActorId, ActorPtr>,
    ref_counter: Map<ActorId, u64>,
}

impl ActorManager {
    fn new() -> ActorManager {
        ActorManager {
            id_counter: AtomicU64::new(0),
            map: Map::with_capacity(20),
            ref_counter: Map::with_capacity(20),
        }
    }

    fn generate_id(&self) -> u64 {
        self.id_counter.fetch_add(1, Ordering::Relaxed)
    }

    fn get_ptr(&self, actor_id: ActorId) -> Option<ActorPtr> {
        self.map.get(&actor_id)
    }

    fn create_actor(&self, size: usize) -> ActorPtr {
        // TODO: properly manage this memory
        let actor_ptr: ActorPtr = ActorPtr(unsafe { libc::malloc(size as libc::size_t) });
        let id = self.generate_id();
        self.map.insert(id, actor_ptr);
        actor_ptr
    }

    fn get_behavior(&self, actor: ActorId, behavior: String) -> Option<ExternalBehavior> {
        // TODO: get behavior
        None
    }
}

struct Message {
    actor_id: ActorId,
    method: String,
    referenced_actors: Vec<ActorId>,
    args: Vec<u8>,
}

struct Scheduler {
    queue: SegQueue<Message>,
    worker_thread_pool: ThreadPool,
    actor_manager: ActorManager,
}

impl Scheduler {
    fn new(worker_thread_count: usize) -> Scheduler {
        Scheduler {
            queue: SegQueue::new(),
            worker_thread_pool: ThreadPoolBuilder::new().num_threads(worker_thread_count).build().unwrap(),
            actor_manager: ActorManager::new(),
        }
    }

    fn add_message(&self, message: Message) {
        self.queue.push(message)
    }

    fn take_message(&self) -> Option<Message> {
        self.queue.pop()
    }

    fn run_next_message(&self) {
        self.worker_thread_pool.install(|| {
            if let Some(message) = self.take_message() {
                if let Some(actor_ptr) = self.actor_manager.get_ptr(message.actor_id) {
                    if let Some(behavior) = self.actor_manager.get_behavior(message.actor_id, message.method) {
                        unsafe {
                            behavior(actor_ptr.0, message.referenced_actors, message.args);
                        }
                    }
                }
            }
        });
    }
}

#[no_mangle]
pub extern "C" fn alox_runtime_queue_message(
    actor_id: ActorId, referenced_actors_length: u8, referenced_actors: *mut ActorId,
    method: *mut libc::c_char, arg_length: u8, args: *mut u8,
) {
    // TODO: properly manage the arrays passed instead of copying all the data
    unsafe {
        // build arg vec
        let raw_args = from_raw_parts(args, arg_length as usize);
        let mut args = Vec::with_capacity(arg_length as usize);
        args.extend_from_slice(raw_args);

        // build referenced actors vec
        let raw_referenced_actors = from_raw_parts(referenced_actors, referenced_actors_length as usize);
        let mut referenced_actors = Vec::with_capacity(referenced_actors_length as usize);
        referenced_actors.extend_from_slice(raw_referenced_actors);

        // get method name
        let method = CString::from_raw(method).to_string_lossy().to_string();

        // build the message and add it to the queue
        let message = Message {
            actor_id,
            method,
            referenced_actors,
            args,
        };

        SCHEDULER.add_message(message);
    }
}

lazy_static! {
    static ref SCHEDULER: Scheduler = Scheduler::new(5);
}

fn main() {
    println!("Hello, world!");
}
