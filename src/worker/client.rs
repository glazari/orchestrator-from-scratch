use crate::task::{Task, TaskEvent};
use tracing::info;

pub struct Client {
    client: reqwest::Client,
    worker: String,
}

#[derive(Debug)]
pub enum Error {
    ErrorReachingWorker(reqwest::Error),
    StatusCodeError(reqwest::StatusCode, String),
    ErrorDecodingResponse(String),
}

type Result<T> = std::result::Result<T, Error>;

impl Client {
    pub fn new(worker: &str) -> Self {
        Client {
            client: reqwest::Client::new(),
            worker: worker.to_string(),
        }
    }

    pub async fn start_task(&self, task_event: &TaskEvent) -> Result<Task> {
        let url = format!("http://{}/tasks", self.worker);
        info!("Sending task to worker: {:?}", url);
        let res = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(task_event)
            .send()
            .await;
        if res.is_err() {
            return Err(Error::ErrorReachingWorker(res.err().unwrap()));
        }
        let res = res.unwrap();
        if !res.status().is_success() {
            let status = res.status();
            let err = res.text().await;
            let err_str = format!("{:?}", err);
            return Err(Error::StatusCodeError(status, err_str));
        }
        let task = res.json::<Task>().await;
        if task.is_err() {
            let err = task.err().unwrap();
            let err_str = format!("{:?}", err);
            return Err(Error::ErrorDecodingResponse(err_str));
        }
        let task = task.unwrap();
        Ok(task)
    }
}
