
extern crate serde;
extern crate ic_stable_structures;

use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{Cell, DefaultMemoryImpl, Storable};
// use ic_stable_structures::{BoundedStorable, MemoryManager, VirtualMemory};
use std::collections::HashMap;
use std::ops::Bound;
use ic_stable_structures::StableBTreeMap;


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

impl Job {
    const BOUND: Bound = Bound::Bounded {
        max_size: 1024,
        is_fixed_size: false,
    };
}

// Implement Storable trait for Job struct
impl Storable for Job {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    
    // const BOUND: ic_stable_structures::storable::Bound;
    
    fn to_bytes_checked(&self) -> Cow<[u8]> {
        let bytes = self.to_bytes();
        if let ic_stable_structures::storable::Bound::Bounded {
            max_size,
            is_fixed_size,
        } = Self::BOUND
        {
            if is_fixed_size {
                assert_eq!(
                    bytes.len(),
                    max_size as usize,
                    "expected a fixed-size element with length {} bytes, but found {} bytes",
                    max_size,
                    bytes.len()
                );
            } else {
                assert!(
                    bytes.len() <= max_size as usize,
                    "expected an element with length <= {} bytes, but found {} bytes",
                    max_size,
                    bytes.len()
                );
            }
        }
        bytes
    }
    
    const BOUND: ic_stable_structures::storable::Bound;
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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Define a struct to represent a job posting
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JobPosting {
    id: u64,
    title: String,
    description: String,
    required_skills: Vec<String>,
}

// Define a struct to represent an employee
// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct Employee {
//     id: u64,
//     name: String,
//     skills: Vec<String>,
// }
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
struct Employee {
    id: u64,
    name: String,
    skills: Vec<String>,
}


// Define a struct to represent a company
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Company {
    id: u64,
    name: String,
    job_postings: Vec<JobPosting>,
}

// Define a struct to represent the government
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Government;

// Define a struct to represent the job matching system
#[derive(Debug)]
struct JobMatcher {
    companies: Vec<Company>,
    employees: Vec<Employee>,
}

impl JobMatcher {
    // Method to add a company to the job matcher
    fn add_company(&mut self, company: Company) {
        self.companies.push(company);
    }

    // Method to add an employee to the job matcher
    fn add_employee(&mut self, employee: Employee) {
        self.employees.push(employee);
    }

    // Method to match employees with suitable jobs
    fn match_jobs(&self) -> HashMap<&Employee, Vec<&JobPosting>> {
        let mut job_matches: HashMap<&Employee, Vec<&JobPosting>> = HashMap::new();

        for employee in &self.employees {
            let mut matches: Vec<&JobPosting> = Vec::new();

            for company in &self.companies {
                for job_posting in &company.job_postings {
                    if job_posting.required_skills.iter().all(|skill: &String| employee.skills.contains(skill))
                    {
                        matches.push(job_posting);
                    }
                }
            }

            job_matches.insert(employee, matches);
        }

        job_matches
    }
}

fn main() {
    // Sample data initialization
    let mut job_matcher = JobMatcher {
        companies: Vec::new(),
        employees: Vec::new(),
    };

    let company1 = Company {
        id: 1,
        name: "Company A".to_string(),
        job_postings: vec![
            JobPosting {
                id: 1,
                title: "Software Engineer".to_string(),
                description: "Develop software applications".to_string(),
                required_skills: vec!["Programming".to_string(), "Problem Solving".to_string()],
            },
            JobPosting {
                id: 2,
                title: "Data Scientist".to_string(),
                description: "Analyze and interpret complex datasets".to_string(),
                required_skills: vec!["Data Analysis".to_string(), "Statistics".to_string()],
            },
        ],
    };

    let employee1 = Employee {
        id: 1,
        name: "John Doe".to_string(),
        skills: vec!["Programming".to_string(), "Problem Solving".to_string()],
    };

    let employee2 = Employee {
        id: 2,
        name: "Jane Doe".to_string(),
        skills: vec!["Data Analysis".to_string(), "Statistics".to_string()],
    };

    job_matcher.add_company(company1.clone());
    job_matcher.add_employee(employee1.clone());
    job_matcher.add_employee(employee2.clone());

    // Match jobs for employees
    let job_matches = job_matcher.match_jobs();
    for (employee, jobs) in job_matches.iter() {
        println!("Employee: {}", employee.name);
        println!("Matched Jobs:");
        for job in jobs {
            println!("  - {}", job.title);
        }
        println!();
    }

}