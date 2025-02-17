#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

// Define Memory and IdCell types
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Define the Job struct
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Job {
    id: u64,
    title: String,
    description: String,
    skills_required: Vec<String>,
    created_at: u64,
}

// Implement Storable trait for Job struct
impl Storable for Job {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement BoundedStorable trait for Job struct
impl BoundedStorable for Job {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Define thread-local variables for memory management and storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, Job, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

// Define JobPayload struct
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct JobPayload {
    title: String,
    description: String,
    skills_required: Vec<String>,
}

// Implementing CRUD functionality for jobs
#[ic_cdk::query]
fn get_job(id: u64) -> Result<Job, Error> {
    match _get_job(&id) {
        Some(job) => Ok(job),
        None => Err(Error::NotFound {
            msg: format!("a job with id={} not found", id),
        }),
    }
}

fn _get_job(id: &u64) -> Option<Job> {
    STORAGE.with(|s| s.borrow().get(id))
}

#[ic_cdk::update]
fn add_job(job: JobPayload) -> Option<Job> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let job = Job {
        id,
        title: job.title,
        description: job.description,
        skills_required: job.skills_required,
        created_at: time(),
    };
    do_insert(&job);
    Some(job)
}

fn do_insert(job: &Job) {
    STORAGE.with(|service| service.borrow_mut().insert(job.id, job.clone()));
}

#[ic_cdk::update]
fn update_job(id: u64, payload: JobPayload) -> Result<Job, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut job) => {
            job.title = payload.title;
            job.description = payload.description;
            job.skills_required = payload.skills_required;
            do_insert(&job);
            Ok(job)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a job with id={}. job not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn delete_job(id: u64) -> Result<Job, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(job) => Ok(job),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a job with id={}. job not found.",
                id
            ),
        }),
    }
}

// Define Error enum
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// Exporting Candid interface definitions
ic_cdk::export_candid!();