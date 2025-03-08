use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub state: State,
    pub image: String,
    pub memory: u64,
    pub disk: u64,
    pub exposed_ports: HashSet<Port>,
    pub port_bindings: HashMap<String, String>,
    pub restart_policy: String,
    pub start_time: DateTime<Utc>,
    pub finish_time: Option<DateTime<Utc>>,
}

impl Default for Task {
    fn default() -> Self {
        Task {
            id: Uuid::new_v4(),
            name: "".to_string(),
            state: State::Pending,
            image: "".to_string(),
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

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
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
