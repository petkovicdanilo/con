use anyhow::Result;
use cgroups_rs::{
    cgroup_builder::CgroupBuilder, cpu::CpuController, memory::MemController, pid::PidController,
    Cgroup, CgroupPid, Controller, MaxValue,
};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Config {
    /// CPU shares (relative weight)
    #[clap(short, long, default_value = "256")]
    pub(crate) cpu_shares: u64,

    /// Memory limit in bytes
    #[clap(short, long, default_value = "1073741824")]
    pub(crate) memory: u64,

    /// Tune container pids limit (0 for unlimited)
    #[clap(short, long, default_value = "0")]
    pub(crate) pids_limit: u32,
}

fn cgroup_name(container_name: &str) -> String {
    format!("con/{}", container_name)
}

pub struct CGroup {
    pub name: String,
    pub inner: Cgroup,
}

impl CGroup {
    pub fn new(container_name: &str, config: &Config) -> Result<Self> {
        let hierarchy = cgroups_rs::hierarchies::auto();
        let name = cgroup_name(&container_name);

        let inner = CgroupBuilder::new(&name)
            .cpu()
            .shares(config.cpu_shares)
            .done()
            .memory()
            .memory_hard_limit(config.memory as i64)
            .done()
            .pid()
            .maximum_number_of_processes(if config.pids_limit == 0 {
                MaxValue::Max
            } else {
                MaxValue::Value(config.pids_limit as i64)
            })
            .done()
            .build(hierarchy);

        Ok(Self { name, inner })
    }

    pub fn add_process(&self, pid: u64) -> Result<()> {
        let pid = CgroupPid::from(pid);

        let cpu_controller: &CpuController = self.inner.controller_of().unwrap();
        cpu_controller.set_notify_on_release(true)?;
        cpu_controller.add_task(&pid)?;

        let memory_controller: &MemController = self.inner.controller_of().unwrap();
        memory_controller.set_notify_on_release(true)?;
        memory_controller.add_task(&pid)?;

        let pids_controller: &PidController = self.inner.controller_of().unwrap();
        pids_controller.set_notify_on_release(true)?;
        pids_controller.add_task(&pid)?;

        Ok(())
    }

    pub fn delete(&mut self) -> Result<()> {
        self.inner.delete()?;

        Ok(())
    }
}
