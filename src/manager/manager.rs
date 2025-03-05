use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use uuid::Uuid;

use crate::task::{Task, TaskEvent};

pub struct Manager {
    // the book still did not make 100% clear what does in the queue
    // is of Tasks, I think it is.
    pending: VecDeque<Task>,
    // once we start using task_db I'll consider not using the Arc
    task_db: HashMap<Uuid, Arc<Task>>,
    event_db: HashMap<Uuid, Vec<TaskEvent>>,
    workers: Vec<String>,
    worker_task_map: HashMap<String, Vec<Uuid>>,
    task_worker_map: HashMap<Uuid, String>,
}

impl Manager {
    pub fn select_worker(&self) -> () {
        println!("I will select an appropriate worker");
    }
    pub fn update_tasks(&self) -> () {
        println!("I will update tasks");
    }
    pub fn send_work() {
        println!("I will send work to a worker");
    }
}
