use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use arc_swap::ArcSwap;
use chrono::Utc;
use tracing::{error, info};
use uuid::Uuid;

use super::stats::{self, Stats};
use crate::task::{self, Task};

#[derive(Debug)]
pub struct Worker {
    pub name: String,
    // maybe use a different data structure here for the queue
    pub queue: Mutex<VecDeque<Task>>,
    pub db: Mutex<HashMap<Uuid, Task>>,
    pub stats: ArcSwap<Stats>,
    pub task_count: u64,
}

impl Worker {
    pub fn new(name: &str) -> Worker {
        Worker {
            name: name.to_string(),
            queue: Mutex::new(VecDeque::new()),
            db: Mutex::new(HashMap::new()),
            stats: ArcSwap::new(Arc::new(stats::get_stats())),
            task_count: 0,
        }
    }

    pub fn add_task(&self, t: Task) {
        // TODO: think of a way to deal with lock errors like this.
        self.queue.lock().unwrap().push_back(t);
    }
    pub async fn run_task(&self) -> task::DockerResult {
        let t = self.queue.lock().unwrap().pop_front();
        if t.is_none() {
            info!("[WORKER] No tasks in queue");
            return task::DockerResult::success("", "", "");
        }

        let t = t.unwrap();

        let persisted_t = {
            let mut db = self.db.lock().unwrap();
            db.entry(t.id).or_insert(t.clone()).clone()
        };

        if !task::is_valid_transition(persisted_t.state, t.state) {
            let err = format!(
                "[WORKER] Invalid state transition from {:?} to {:?}\n",
                persisted_t.state, t.state
            );
            error!("{}", err);
            return task::DockerResult::error(&err);
        }

        match t.state {
            task::State::Scheduled => self.start_task(t).await,
            task::State::Completed => self.stop_task(t).await,
            _ => {
                error!("[WORKER] Invalid state transition to {:?}", t.state);
                task::DockerResult::error("Invalid state transition")
            }
        }
    }
    pub async fn start_task(&self, mut t: Task) -> task::DockerResult {
        t.start_time = Utc::now();
        let config = task::new_config(&t);
        let d = task::new_docker(config);

        let result = d.run().await;
        if result.error.is_some() {
            let error = result.error.as_ref().unwrap();
            error!("[WORKER] Error running task {:?}: {}", t.id, error);
            t.state = task::State::Failed;
            self.db.lock().unwrap().insert(t.id, t);
            return result;
        }

        t.container_id = result.container_id.clone();
        t.state = task::State::Running;
        self.db.lock().unwrap().insert(t.id, t);
        return result;
    }
    pub async fn stop_task(&self, mut t: Task) -> task::DockerResult {
        let config = task::new_config(&t);
        let d = task::new_docker(config);

        let result = d.stop(&t.container_id).await;
        if result.error.is_some() {
            error!("[WORKER] Error stopping task: {:?}", result.error.as_ref().unwrap());
        }
        t.finish_time = Some(Utc::now());
        t.state = task::State::Completed;
        error!(
            "[WORKER] Stopped and removed container {:?} for task {:?}",
            t.container_id, t.id
        );
        self.db.lock().unwrap().insert(t.id, t);
        return result;
    }
}

pub async fn collect_stats(worker: Arc<Worker>) -> () {
    loop {
        info!("[WORKER] Collecting stats");
        let stats = stats::get_stats();
        worker.stats.store(Arc::new(stats));
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
