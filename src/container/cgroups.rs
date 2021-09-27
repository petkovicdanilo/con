use anyhow::Result;
use cgroups_rs::{
    cgroup_builder::CgroupBuilder, cpu::CpuController, memory::MemController, pid::PidController,
    Cgroup, CgroupPid, Controller, MaxValue,
};
use clap::Clap;
use nix::unistd::getpid;

#[derive(Clap, Debug)]
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

pub fn run(config: &Config, container_name: &str) -> Result<()> {
    let hierarchy = cgroups_rs::hierarchies::auto();

    let cgroup = CgroupBuilder::new(&cgroup_name(&container_name))
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

    let pid = CgroupPid::from(getpid().as_raw() as u64);

    let cpu_controller: &CpuController = cgroup.controller_of().unwrap();
    cpu_controller.set_notify_on_release(true)?;
    cpu_controller.add_task(&pid)?;

    let memory_controller: &MemController = cgroup.controller_of().unwrap();
    memory_controller.set_notify_on_release(true)?;
    memory_controller.add_task(&pid)?;

    let pids_controller: &PidController = cgroup.controller_of().unwrap();
    pids_controller.set_notify_on_release(true)?;
    pids_controller.add_task(&pid)?;

    Ok(())
}

pub fn remove_cgroups(container_name: &str) -> Result<()> {
    let hierarchy = cgroups_rs::hierarchies::auto();

    let cgroup = Cgroup::load(hierarchy, cgroup_name(container_name));
    cgroup.delete()?;

    Ok(())
}
