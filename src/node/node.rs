#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub ip: String,
    pub cores: u32,
    pub memory: u64,
    pub memory_allocated: u64,
    pub disk: u64,
    pub disk_allocated: u64,
    pub role: String,
    pub task_count: u32,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            name: "".to_string(),
            ip: "".to_string(),
            cores: 0,
            memory: 0,
            memory_allocated: 0,
            disk: 0,
            disk_allocated: 0,
            role: "".to_string(),
            task_count: 0,
        }
    }
}
