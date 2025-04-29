use std::collections::{HashMap, VecDeque};
use tokio::sync::Mutex;

use tracing::info;
use uuid::Uuid;

use crate::task::{self, Task, TaskEvent};

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
    pub async fn select_worker(&self) -> String {
        let mut value = self.last_worker.lock().await;
        let new_value = (*value + 1) % self.workers.len();
        *value = new_value;
        self.workers[new_value].clone()
    }
    pub async fn update_tasks(&self) -> () {
        for worker in &self.workers {
            println!("Checking worker {} for task updates", worker);
            let url = format!("http://{}/tasks", worker);
            let client = reqwest::Client::new();
            let res = client.get(&url).send().await;

            if res.is_err() {
                println!("Error connecting to {}: {}", worker, res.err().unwrap());
                continue;
            }
            let res = res.unwrap();

            if !res.status().is_success() {
                println!("Error getting tasks from {}: {}", worker, res.status());
                continue;
            }

            let tasks = res.json::<Vec<Task>>().await;
            if tasks.is_err() {
                println!("Error decoding response: {:?}", tasks.err());
                continue;
            }

            let tasks = tasks.unwrap();
            let mut task_db = self.task_db.lock().await;
            for task in tasks {
                println!("Attempting to update task {}", task.id);

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
            println!("No tasks in queue");
            return;
        }
        let te = e.unwrap();
        let task = task::Task {
            state: task::State::Scheduled,
            ..te.task.clone()
        };

        let w = self.select_worker().await;

        info!("pulled {:?} from queue", task);

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
        let data = serde_json::to_string(&task).expect("Failed to serialize task");
        let client = reqwest::Client::new();
        let res = client
            .post(&format!("http://{}/task", w))
            .header("Content-Type", "application/json")
            .body(data)
            .send()
            .await;
        if res.is_err() {
            println!("Error sending task to worker: {:?}", res.err());
            self.pending.lock().await.push_back(te);
            return;
        }
        let res = res.unwrap();
        if !res.status().is_success() {
            println!("Error sending task to worker: {:?}", res.status());
            let err = res.text().await;
            println!("Error sending task to worker: {:?}", err);
            return;
        }
        let task = res.json::<Task>().await;
        if task.is_err() {
            println!("Error decoding response: {:?}", task.err());
            return;
        }
        let task = task.unwrap();
        println!("Task sent to worker: {:?}", task);
    }

    fn new(workers: Vec<&str>) -> Self {
        let workers = workers
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
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
