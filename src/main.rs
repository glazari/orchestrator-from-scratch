use std::collections::{HashMap, VecDeque};
use std::default::Default;

use chrono::Utc;
use uuid::Uuid;

use cube::manager;
use cube::node;
use cube::task;
use cube::worker;

fn main() {
    println!("Hello, world!");
    let task = task::Task {
        id: Uuid::new_v4(),
        name: "Task 1".to_string(),
        state: task::State::Pending,
        image: "Image 1".to_string(),
        memory: 1024,
        disk: 1,
        ..Default::default()
    };

    let task_event = task::TaskEvent {
        id: Uuid::new_v4(),
        state: task::State::Pending,
        timestamp: Utc::now(),
        task: task.clone(),
    };

    println!("{:#?}", task);
    println!("{:#?}", task_event);

    let worker = worker::Worker {
        name: "Worker 1".to_string(),
        queue: VecDeque::new(),
        db: HashMap::new(),
        task_count: 0,
    };

    println!("{:#?}", worker);
    worker.collect_stats();
    worker.run_task();
    worker.start_task();
    worker.stop_task();

    let manager = manager::Manager {
        pending: VecDeque::new(),
        task_db: HashMap::new(),
        event_db: HashMap::new(),
        workers: Vec::new(),
        worker_task_map: HashMap::new(),
        task_worker_map: HashMap::new(),
    };

    println!("{:#?}", manager);
    manager.select_worker();
    manager.update_tasks();
    manager.send_work();

    let node = node::Node {
        name: "Node 1".to_string(),
        ip: "192.168.1.1".to_string(),
        cores: 4,
        memory: 1024,
        disk: 25,
        role: "Worker".to_string(),
        ..Default::default()
    };

    println!("{:#?}", node);
}
