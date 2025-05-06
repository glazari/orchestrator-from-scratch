use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use tracing::info;
use uuid::Uuid;

use super::manager::{self, Manager};
use crate::task::{self, Task, TaskEvent};

type AppState = State<Arc<Manager>>;

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

pub async fn start_api(api: Api, manager: Arc<Manager>) {
    tokio::spawn(manager::process_tasks(manager.clone()));
    tokio::spawn(manager::update_tasks_loop(manager.clone()));
    api.start().await;
}

pub fn setup(address: &str, port: u16, manager: Arc<Manager>) -> Api {
    let router = Router::new()
        .route("/tasks", post(start_task_handler))
        .route("/tasks", get(get_tasks))
        .route("/tasks/{task_id}", delete(stop_task))
        .with_state(manager);
    Api {
        address: address.to_string(),
        port,
        router,
    }
}

// TODO have a default 400 response for all routes
async fn start_task_handler(
    State(manager): AppState,
    Json(te): Json<TaskEvent>,
) -> (StatusCode, Json<Task>) {
    manager.add_task(te.clone()).await;
    info!("[MANAGER] Added task {:?}", te.task.id);
    (StatusCode::CREATED, Json(te.task))
}

async fn get_tasks(State(manager): AppState) -> Json<Vec<Task>> {
    Json(manager.get_tasks().await)
}

async fn stop_task(
    State(manager): AppState,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let task_db = manager.task_db.lock().await;
    let task_to_stop = task_db.get(&task_id);

    if task_to_stop.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let mut task_copy = task_to_stop.unwrap().clone();
    task_copy.state = task::State::Completed;
    let task_id = task_copy.id.clone();

    let task_event = TaskEvent {
        id: Uuid::new_v4(),
        state: task::State::Completed,
        timestamp: chrono::Utc::now(),
        task: task_copy,
        ..Default::default()
    };

    let te_id = task_event.id.clone();

    manager.add_task(task_event).await;

    info!(
        "[MANAGER] Added event {:?} to stop task {:?}",
        te_id, task_id
    );

    Ok(StatusCode::NO_CONTENT)
}
