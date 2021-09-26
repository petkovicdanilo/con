use anyhow::Result;
use cgroups_rs::{cgroup_builder::CgroupBuilder, CgroupPid, MaxValue};
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

const CGROUP_NAME: &str = "con";

pub fn run(config: &Config) -> Result<()> {
    let hierarchy = cgroups_rs::hierarchies::auto();

    let cgroup = CgroupBuilder::new(CGROUP_NAME)
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

    // automatically delete cgroup after process exits
    cgroup.set_notify_on_release(true)?;

    let pid = getpid().as_raw() as u64;
    cgroup.add_task(CgroupPid::from(pid))?;

    Ok(())
}
