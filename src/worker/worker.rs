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
    pub fn new(name: &str) -> Worker {
        Worker {
            name: name.to_string(),
            queue: VecDeque::new(),
            db: HashMap::new(),
            task_count: 0,
        }
    }

    pub fn collect_stats(&self) -> () {
        println!("I will collect stats");
    }
    pub fn add_task(&mut self, t: Task) {
        self.queue.push_back(t);
    }
    pub async fn run_task(&mut self) -> task::DockerResult {
        let t = self.queue.pop_front();
        if t.is_none() {
            println!("No tasks in queue");
            return task::DockerResult {
                action: "".to_string(),
                container_id: "".to_string(),
                error: None,
                result: "".to_string(),
            };
        }

        let t = t.unwrap();

        let persisted_t = self.db.entry(t.id).or_insert(t.clone());

        if !task::is_valid_transition(persisted_t.state, t.state) {
            let err = format!(
                "Invalid state transition from {:?} to {:?}\n",
                persisted_t.state, t.state
            );
            println!("{}", err);
            return task::DockerResult {
                action: "".to_string(),
                container_id: "".to_string(),
                error: Some(err),
                result: "".to_string(),
            };
        }

        match t.state {
            task::State::Scheduled => self.start_task(t).await,
            task::State::Completed => self.stop_task(t).await,
            _ => {
                println!("Invalid state transition");
                task::DockerResult {
                    action: "".to_string(),
                    container_id: "".to_string(),
                    error: Some("Invalid state transition".to_string()),
                    result: "".to_string(),
                }
            }
        }
    }
    pub async fn start_task(&mut self, mut t: Task) -> task::DockerResult {
        t.start_time = Utc::now();
        let config = task::new_config(&t);
        let d = task::new_docker(config);

        let result = d.run().await;
        if result.error.is_some() {
            let error = result.error.as_ref().unwrap();
            println!("Error running task {:?}: {}", t.id, error);
            t.state = task::State::Failed;
            self.db.insert(t.id, t);
            return result;
        }

        t.container_id = result.container_id.clone();
        t.state = task::State::Running;
        self.db.insert(t.id, t);
        return result;
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
