use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display, Formatter};

use bollard::container::{self, LogsOptions};
use bollard::image::CreateImageOptions;
use bollard::models::{CreateImageInfo, HostConfig, RestartPolicy};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub container_id: String,
    pub name: String,
    pub state: State,
    pub image: String,
    pub cpu: f64,
    pub memory: u64,
    pub disk: u64,
    pub exposed_ports: HashSet<Port>,
    pub port_bindings: HashMap<String, String>,
    pub restart_policy: String, // empty, always, unless-stopped, on-failure
    pub start_time: DateTime<Utc>,
    pub finish_time: Option<DateTime<Utc>>,
}

impl Default for Task {
    fn default() -> Self {
        Task {
            id: Uuid::new_v4(),
            container_id: "".to_string(),
            name: "".to_string(),
            state: State::Pending,
            image: "".to_string(),
            cpu: 0.0,
            memory: 0,
            disk: 0,
            exposed_ports: HashSet::new(),
            port_bindings: HashMap::new(),
            restart_policy: "".to_string(),
            start_time: Utc::now(),
            finish_time: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Pending,
    Scheduled,
    Running,
    Completed,
    Failed,
}

// for now, defining my own port struct
// if it turns out we need more sofisticated functionality we can look for a library
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct Port {
    pub number: u16,
    pub protocol: Protocol,
}

impl Port {
    pub fn to_docker_repr(&self) -> (String, HashMap<(), ()>) {
        let port = format!("{}/{}", self.number, self.protocol);
        (port, HashMap::new())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::Udp => write!(f, "udp"),
        }
    }
}

#[derive(Debug)]
pub struct TaskEvent {
    pub id: Uuid,
    pub state: State,
    pub timestamp: DateTime<Utc>,
    pub task: Task, // TODO: check if this will be a copy of the task or if the idea is
                    // to modify the task in place :fearful:
}

impl Default for TaskEvent {
    fn default() -> Self {
        TaskEvent {
            id: Uuid::new_v4(),
            state: State::Pending,
            timestamp: Utc::now(),
            task: Task::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub name: String,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub attach_stderr: bool,
    pub exposed_ports: HashSet<Port>,
    pub cmd: Vec<String>,
    pub image: String,
    pub cpu: f64,
    pub memory: i64,
    pub disk: u64,
    pub env: Vec<String>,       // maybe use a pair?
    pub restart_policy: String, // empty, always, unless-stopped, on-failure
}

pub fn new_config(t: &Task) -> Config {
    Config {
        name: t.name.clone(),
        exposed_ports: t.exposed_ports.clone(),
        image: t.image.clone(),
        cpu: t.cpu,
        memory: t.memory as i64,
        disk: t.disk,
        restart_policy: t.restart_policy.clone(),
        ..Default::default()
    }
}

pub struct Docker {
    pub client: bollard::Docker,
    pub config: Config,
}

pub fn new_docker(config: Config) -> Docker {
    let client = bollard::Docker::connect_with_local_defaults().unwrap();
    Docker { client, config }
}

impl Docker {
    pub async fn run(&self) -> DockerResult {
        let image = self.config.image.split(":").collect::<Vec<_>>();
        let options = CreateImageOptions {
            from_image: image[0],
            tag: image[1],

            ..Default::default()
        };
        let root_fs = None;
        let credentials = None;
        let mut res = self
            .client
            .create_image(Some(options), root_fs, credentials);

        // un stream
        let single_result: Option<Result<CreateImageInfo, bollard::errors::Error>> =
            res.next().await;
        let single_result = single_result.map_or_else(
            || {
                Err(bollard::errors::Error::DockerResponseServerError {
                    status_code: 500,
                    message: "No result".to_string(),
                })
            },
            |x| x,
        );

        let single_result = match single_result {
            Ok(x) => x,
            Err(e) => {
                return DockerResult {
                    error: Some(e.to_string()),
                    action: "run".to_string(),
                    container_id: "".to_string(),
                    result: "".to_string(),
                };
            }
        };

        println!("{:?}", single_result);
        // TODO: do something with the result
        //

        let restart_policy_name = self
            .config
            .restart_policy
            .parse()
            .expect("Invalid restart policy");

        let rp = RestartPolicy {
            name: Some(restart_policy_name),
            ..RestartPolicy::default()
        };

        let r = HostConfig {
            memory: Some(self.config.memory),
            nano_cpus: Some((self.config.cpu * 1_000_000_000.0) as i64),
            restart_policy: Some(rp),
            publish_all_ports: Some(true),
            ..HostConfig::default()
        };

        let cc = container::Config {
            image: Some(self.config.image.clone()),
            tty: Some(false),
            env: Some(self.config.env.clone()),
            exposed_ports: Some(
                self.config
                    .exposed_ports
                    .iter()
                    .map(|x| x.to_docker_repr())
                    .collect::<_>(),
            ),
            host_config: Some(r),
            ..container::Config::default()
        };

        // TODO: create container
        let options = container::CreateContainerOptions {
            name: self.config.name.clone(),
            //TODO: maybe add a platform
            platform: None,
        };

        let res = self.client.create_container(Some(options), cc).await;
        let res = match res {
            Ok(x) => x,
            Err(e) => {
                return DockerResult {
                    error: Some(e.to_string()),
                    action: "run".to_string(),
                    container_id: "".to_string(),
                    result: "".to_string(),
                };
            }
        };

        println!("container create res: {:?}", res);

        let err = self.client.start_container::<String>(&res.id, None).await;
        if let Err(e) = err {
            return DockerResult {
                error: Some(e.to_string()),
                action: "run".to_string(),
                container_id: "".to_string(),
                result: "".to_string(),
            };
        }

        let container_id = res.id.clone();

        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            ..LogsOptions::default()
        };
        let mut res = self.client.logs(&res.id, Some(options));

        let mut count = 0;
        while let Some(res) = res.next().await {
            let res = match res {
                Ok(x) => x,
                Err(e) => {
                    return DockerResult {
                        error: Some(e.to_string()),
                        action: "run".to_string(),
                        container_id: "".to_string(),
                        result: "".to_string(),
                    };
                }
            };

            println!("logs: {:?}", res);
            count += 1;
            if count > 10 {
                break;
            }
        }

        return DockerResult {
            error: None,
            action: "start".to_string(),
            container_id,
            result: "success".to_string(),
        };
    }

    pub async fn stop(&self, id: &str) -> DockerResult {
        println!("stopping container: {}", id);
        let res = self.client.stop_container(&id, None).await;
        let res = match res {
            Ok(x) => x,
            Err(e) => {
                return DockerResult {
                    error: Some(e.to_string()),
                    action: "stop".to_string(),
                    container_id: id.to_string(),
                    result: "".to_string(),
                };
            }
        };

        println!("container stop res: {:?}", res);

        let options = container::RemoveContainerOptions {
            v: true,
            link: false,
            force: false,
        };
        let res = self.client.remove_container(&id, Some(options)).await;
        if let Err(e) = res {
            return DockerResult {
                error: Some(e.to_string()),
                action: "stop".to_string(),
                container_id: id.to_string(),
                result: "".to_string(),
            };
        }

        return DockerResult {
            error: None,
            action: "stop".to_string(),
            container_id: id.to_string(),
            result: "success".to_string(),
        };
    }
}

pub struct DockerResult {
    pub error: Option<String>,
    pub action: String,
    pub container_id: String,
    pub result: String,
}
