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
    number: u16,
    protocol: Protocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

pub struct TaskEvent {
    id: Uuid,
    state: State,
    timestamp: DateTime<Utc>,
    task: Task, // TODO: check if this will be a copy of the task or if the idea is
                // to modify the task in place :fearful:
}
