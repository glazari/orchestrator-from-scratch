use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use uuid::Uuid;

use crate::task::{Task, self};

#[derive(Debug)]
pub struct Worker {
    pub name: String,
    // I think this is a queue of tasks, the book did not say yet.
    pub queue: VecDeque<Task>,
    // The book puts Task behind a pointer, I'm not sure if that is done
    // for compactness of the map, or its done to allow for shared ownership
    // and modification of the task. I'm going to assume the latter.
    pub db: HashMap<Uuid, Arc<Task>>,
    pub task_count: u64,
}

impl Worker {
    pub fn collect_stats(&self) -> () {
        println!("I will collect stats");
    }
    pub fn run_task(&self) -> () {
        println!("I will start or stop a task");
    }
    pub fn start_task(&self) -> () {
        println!("I will start a task");
    }
    pub fn stop_task(&self) -> () {
        //let config = task::new_config(&t);
        //let d = task::new_docker(config);

       // let result = d.stop().await;
    }
}
