use std::collections::{HashMap, VecDeque};
use std::default::Default;
use std::time::Duration;

use chrono::Utc;
use uuid::Uuid;

use cube::manager;
use cube::node;
use cube::task;
use cube::worker;

#[tokio::main]
async fn main() {
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

    println!("Creating container");
    let (mut docker, result) = create_container().await;
    if result.error.is_some() {
        panic!("Error: {}", result.error.unwrap());
    }

    tokio::time::sleep(Duration::from_secs(5)).await;

    println!("Stopping container");
    let _result = stop_container(&mut docker, &result.container_id).await;
}

async fn create_container() -> (task::Docker, task::DockerResult) {
    let c = task::Config {
        name: "test-container-1".to_string(),
        image: "postgres:latest".to_string(),
        env: vec![
            "POSTGRES_USER=cube".to_string(),
            "POSTGRES_PASSWORD=secret".to_string(),
        ],
        ..Default::default()
    };

    let dc = bollard::Docker::connect_with_local_defaults().expect("Could not connect to docker");
    let mut docker = task::Docker {
        client: dc,
        config: c,
    };

    let result = docker.run().await;
    if result.error.is_some() {
        panic!("Error: {}", result.error.unwrap());
    }

    println!(
        "Container {} is running with config {:?}",
        result.container_id, docker.config
    );

    (docker, result)
}

async fn stop_container(docker: &mut task::Docker, id: &str) -> task::DockerResult {
    let result = docker.stop(id).await;
    if result.error.is_some() {
        panic!("Error: {}", result.error.unwrap());
    }

    println!("Container {} stopped", id);

    result
}
