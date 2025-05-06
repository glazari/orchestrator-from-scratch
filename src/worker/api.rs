use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use tracing::info;
use uuid::Uuid;

use super::stats::Stats;
use super::worker::{self, Worker};
use crate::task::{self, Task, TaskEvent};

type AppState = State<Arc<Worker>>;

pub struct Api {
    address: String,
    port: u16,
    router: Router,
}

impl Api {
    pub async fn start(self) {
        let socket = format!("{}:{}", self.address, self.port);
        let listener = tokio::net::TcpListener::bind(socket).await.unwrap();
        axum::serve(listener, self.router).await.unwrap();
    }
}

pub async fn start_api(api: Api, worker: Arc<Worker>) {
    tokio::spawn(worker::collect_stats(worker.clone()));
    tokio::spawn(worker::run_tasks_loop(worker.clone()));

    api.start().await;
}

pub fn setup(address: &str, port: u16, worker: Arc<Worker>) -> Api {
    let router = Router::new()
        .route("/tasks", post(start_task))
        .route("/tasks", get(get_task))
        .route("/tasks/{task_id}", delete(stop_task))
        .route("/stats", get(get_stats))
        .with_state(worker);
    Api {
        address: address.to_string(),
        port,
        router,
    }
}

// TODO have a default 400 response for all routes
async fn start_task(State(w): AppState, Json(te): Json<TaskEvent>) -> (StatusCode, Json<Task>) {
    w.add_task(te.task.clone());
    info!("[WORKER] Added task {:?}", te.task.id);
    (StatusCode::CREATED, Json(te.task))
}

async fn get_task(State(w): AppState) -> Json<Vec<Task>> {
    let tasks = {
        let db = w.db.lock().expect("Failed to lock worker db");
        db.values().cloned().collect::<Vec<Task>>()
    };
    info!("[WORKER] Getting tasks {:?}", tasks);
    Json(tasks.clone())
}

async fn stop_task(
    State(w): AppState,
    Path(task_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let task_id = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut task_to_stop = {
        let db = w.db.lock().expect("Failed to lock worker db");
        db.get(&task_id).cloned().ok_or(StatusCode::NOT_FOUND)?
    };

    task_to_stop.state = task::State::Completed;

    let (id, container_id) = (&task_to_stop.id, &task_to_stop.container_id);
    info!(
        "[WORKER] added task {:?} to stop container {:?}",
        id, container_id
    );
    w.add_task(task_to_stop);

    Ok(StatusCode::NO_CONTENT)
}

async fn get_stats(State(w): AppState) -> Json<Stats> {
    let stats = w.stats.load().as_ref().clone();
    info!("[WORKER] Getting stats {:?}", stats);
    Json(stats)
}
