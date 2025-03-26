use std::collections::{HashMap, VecDeque};

use chrono::Utc;
use uuid::Uuid;

use crate::task::{self, Task};

#[derive(Debug)]
pub struct Worker {
    pub name: String,
    // I think this is a queue of tasks, the book did not say yet.
    pub queue: VecDeque<Task>,
    // The book puts Task behind a pointer, I'm not sure if that is done
    // for compactness of the map, or its done to allow for shared ownership
    // and modification of the task. I'm going to assume the latter.
    pub db: HashMap<Uuid, Task>,
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
    pub async fn stop_task(&mut self, mut t: Task) -> task::DockerResult {
        let config = task::new_config(&t);
        let d = task::new_docker(config);

        let result = d.stop(&t.container_id).await;
        if result.error.is_some() {
            println!("Error stopping task: {:?}", result.error.as_ref().unwrap());
        }
        t.finish_time = Some(Utc::now());
        t.state = task::State::Completed;
        println!(
            "Stopped and removed container {:?} for task {:?}",
            t.container_id, t.id
        );
        self.db.insert(t.id, t);
        return result;
    }
}
