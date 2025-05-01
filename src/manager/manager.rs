use std::collections::{HashMap, VecDeque};
use tokio::sync::Mutex;

use tracing::{error, info};
use uuid::Uuid;

use crate::task::{self, Task, TaskEvent};
use crate::worker;

#[derive(Debug)]
pub struct Manager {
    // Warning: many Mutexes are in use here. If more than one of them need to
    // be locked at the same time, lock them exactly in the order they appear here.
    // This is to avoid deadlocks.
    // If this situation becomes to common we consider using a single mutex for multiple fields.
    pub pending: Mutex<VecDeque<TaskEvent>>,
    pub task_db: Mutex<HashMap<Uuid, Task>>,
    pub event_db: Mutex<HashMap<Uuid, TaskEvent>>,
    pub workers: Vec<String>,
    pub worker_task_map: Mutex<HashMap<String, Vec<Uuid>>>,
    pub task_worker_map: Mutex<HashMap<Uuid, String>>,
    pub last_worker: Mutex<usize>, // to keep track of the last worker used
}

impl Manager {
    // Warning, if any of these methods call one another, they might take more than
    // one lock at a time. They need to either respect the order of locking described above
    // or not call each other at all.
    pub async fn add_task(&self, te: TaskEvent) -> () {
        let mut pending = self.pending.lock().await;
        pending.push_back(te);
    }
    pub async fn get_tasks(&self) -> Vec<Task> {
        let task_db = self.task_db.lock().await;
        task_db.values().cloned().collect()
    }
    pub async fn select_worker(&self) -> String {
        let mut value = self.last_worker.lock().await;
        let new_value = (*value + 1) % self.workers.len();
        *value = new_value;
        self.workers[new_value].clone()
    }
    pub async fn update_tasks(&self) -> () {
        for worker in &self.workers {
            info!("[MANAGER] Checking worker {} for task updates", worker);
            let url = format!("http://{}/tasks", worker);
            let client = reqwest::Client::new();
            let res = client.get(&url).send().await;

            if res.is_err() {
                let err = res.err().unwrap();
                error!("[MANAGER] Error connecting to {}: {}", worker, err);
                continue;
            }
            let res = res.unwrap();

            if !res.status().is_success() {
                let sts = res.status();
                error!("[MANAGER] Error getting tasks from {}: {}", worker, sts);
                continue;
            }

            let tasks = res.json::<Vec<Task>>().await;
            if tasks.is_err() {
                error!("[MANAGER] Error decoding response: {:?}", tasks.err());
                continue;
            }

            let tasks = tasks.unwrap();
            let mut task_db = self.task_db.lock().await;
            for task in tasks {
                info!("[MANAGER] Attempting to update task {}", task.id);

                let db_task = task_db.get_mut(&task.id);
                db_task.map(|t| {
                    t.state = task.state.clone();
                    t.start_time = task.start_time.clone();
                    t.finish_time = task.finish_time.clone();
                    t.container_id = task.container_id.clone();
                });
            }
        }
    }
    pub async fn send_work(&self) -> () {
        // holds the lock only for this line
        let e = self.pending.lock().await.pop_front();
        if e.is_none() {
            info!("[MANAGER] No tasks in queue");
            return;
        }
        let te = e.unwrap();
        let task = task::Task {
            state: task::State::Scheduled,
            ..te.task.clone()
        };

        let w = self.select_worker().await;

        info!("[MANAGER] pulled {:?} from queue", task);

        // transactional like update. This potentially holds the
        // lock for longer than its needed, but at least we dont worry about
        // inconsistent state.
        {
            let mut task_db = self.task_db.lock().await;
            let mut event_db = self.event_db.lock().await;
            let mut worker_task_map = self.worker_task_map.lock().await;
            let mut task_worker_map = self.task_worker_map.lock().await;

            event_db.insert(te.id, te.clone());
            worker_task_map
                .entry(w.clone())
                .or_insert(vec![])
                .push(task.id);
            task_worker_map.insert(task.id, w.clone());
            task_db.insert(task.id, task.clone());
        }

        // send the task to the worker
        let client = worker::Client::new(&w);
        let res = client.start_task(&te).await;

        let task = match res {
            Ok(task) => task,
            Err(e) => match e {
                // Only if we don't reach the worker we will retry, otherwise we log the error and
                // give up.
                worker::client::Error::ErrorReachingWorker(e) => {
                    error!("[MANAGER] Error reaching worker: {:?}", e);
                    self.pending.lock().await.push_back(te);
                    return;
                }
                _ => {
                    error!("[MANAGER] Error sending task to worker: {:?}", e);
                    return;
                }
            },
        };
        info!("[MANAGER] Task sent to worker: {:?}", task);
    }

    pub fn new(workers: Vec<String>) -> Self {
        let worker_task_map = workers
            .iter()
            .map(|s| (s.clone(), Vec::new()))
            .collect::<HashMap<String, Vec<Uuid>>>();

        Self {
            pending: Mutex::new(VecDeque::new()),
            task_db: Mutex::new(HashMap::new()),
            event_db: Mutex::new(HashMap::new()),
            workers,
            worker_task_map: Mutex::new(worker_task_map),
            task_worker_map: Mutex::new(HashMap::new()),
            last_worker: Mutex::new(0),
        }
    }
}
