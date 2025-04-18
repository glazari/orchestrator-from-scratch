use std::collections::{HashMap, VecDeque};
use std::default::Default;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use cube::manager;
use cube::node;
use cube::task;
use cube::worker;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let host = "localhost";
    let port = 8901;
    info!("Starting Cube worker on {}:{}", host, port);

    let worker = worker::Worker::new("Worker 1");
    let worker = Arc::new(worker);
    let api = worker::api::setup(host, port, worker.clone());

    tokio::spawn(worker::collect_stats(worker.clone()));
    tokio::spawn(run_tasks(worker));
    api.start().await;
}

async fn run_tasks(w: Arc<worker::Worker>) {
    let delay = Duration::from_secs(10);
    loop {
        info!("Sleeping for {} seconds", delay.as_secs());
        tokio::time::sleep(delay).await;

        let len = w.queue.lock().unwrap().len();

        if len == 0 {
            info!("No tasks in queue");
            continue;
        }

        let result = w.run_task().await;
        if result.error.is_some() {
            info!("Error Running task: {}", result.error.unwrap());
        }
    }
}

#[allow(dead_code)]
async fn main_old2() {
    let w = worker::Worker::new("Worker 1");

    let mut t = task::Task {
        id: Uuid::new_v4(),
        name: "test-container-1".to_string(),
        state: task::State::Scheduled,
        image: "strm/helloworld-http".to_string(),
        ..Default::default()
    };

    println!("starting task");
    w.add_task(t.clone());
    let result = w.run_task().await;
    if result.error.is_some() {
        panic!("Error: {}", result.error.unwrap());
    }

    t.container_id = result.container_id;
    println!("task {} is running in container {} ", t.id, t.container_id);
    println!("Sleepy time");
    tokio::time::sleep(Duration::from_secs(30)).await;

    println!("stopping task");
    t.state = task::State::Completed;
    w.add_task(t.clone());
    let result = w.run_task().await;
    if result.error.is_some() {
        panic!("Error: {}", result.error.unwrap());
    }
}

#[allow(dead_code)]
async fn main_old() {
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

    let worker = worker::Worker::new("Worker 1");
    let worker = Arc::new(worker);

    println!("{:#?}", worker);
    worker::collect_stats(worker.clone()).await;
    worker.run_task().await;
    //worker.start_task(task.clone()).await;
    //worker.stop_task(task).await;

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
    let docker = task::Docker {
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
