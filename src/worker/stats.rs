use procfs::prelude::*;
use procfs::{CpuInfo, CpuPressure, CpuTime, KernelStats, LoadAverage, LocalSystemInfo, Meminfo};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Stats {
    pub mem_stats: Meminfo,
    pub disk_stats: Vec<DiskStats>,
    pub cpu_info: CpuInfo,
    pub cpu_pressure: CpuPressure,
    pub cpu_time: CpuTime,
    pub load_avg: LoadAverage,
    pub task_count: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct DiskStats {
    pub total: u64,
    pub free: u64,
    pub mount_point: String,
    pub file_system: String,
}

impl From<&sysinfo::Disk> for DiskStats {
    fn from(disk: &sysinfo::Disk) -> Self {
        DiskStats {
            total: disk.total_space(),
            free: disk.available_space(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            file_system: disk.file_system().to_string_lossy().to_string(),
        }
    }
}

impl Stats {
    // memory stats

    pub fn mem_total_kb(&self) -> u64 {
        self.mem_stats.mem_total
    }

    pub fn mem_available_kb(&self) -> u64 {
        self.mem_stats.mem_free
    }

    pub fn mem_used_kb(&self) -> u64 {
        self.mem_total_kb() - self.mem_available_kb()
    }

    pub fn mem_used_percent(&self) -> f32 {
        (self.mem_used_kb() as f32 / self.mem_total_kb() as f32) * 100.0
    }

    // disk stats

    pub fn disk_total_bytes(&self) -> u64 {
        self.disk_stats.iter().map(|disk| disk.total).sum()
    }

    pub fn disk_free_bytes(&self) -> u64 {
        self.disk_stats.iter().map(|disk| disk.free).sum()
    }

    pub fn disk_used_bytes(&self) -> u64 {
        self.disk_total_bytes() - self.disk_free_bytes()
    }

    // CPU stats
    pub fn cpu_usage(&self) -> f32 {
        let d = &self.cpu_time;
        let idle = d.idle + d.iowait.unwrap_or(0);
        let non_idle = d.user
            + d.nice
            + d.system
            + d.irq.unwrap_or(0)
            + d.softirq.unwrap_or(0)
            + d.steal.unwrap_or(0);
        let total = idle + non_idle;

        if total == 0 {
            return 0.0;
        }

        (non_idle as f32 / total as f32) * 100.0
    }
}

pub fn get_stats() -> Stats {
    let mem_stats = Meminfo::from_file("/proc/meminfo").unwrap();
    let disk_stats = sysinfo::Disks::new_with_refreshed_list();
    let disk_stats = disk_stats.iter().map(|d| d.into()).collect();

    let cpu_info = CpuInfo::from_file("/proc/cpuinfo").unwrap();
    let cpu_pressure = CpuPressure::from_file("/proc/pressure/cpu").unwrap();
    let cpu_time = KernelStats::from_file("/proc/stat", &LocalSystemInfo)
        .unwrap()
        .total;
    let load_avg = procfs::LoadAverage::from_file("/proc/loadavg").unwrap();

    Stats {
        mem_stats,
        disk_stats,
        cpu_info,
        cpu_pressure,
        cpu_time,
        load_avg,
        task_count: 0,
    }
}
