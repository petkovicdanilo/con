use anyhow::Result;
use cgroups_rs::{cgroup_builder::CgroupBuilder, CgroupPid, MaxValue};
use nix::unistd::getpid;

use crate::commands::run::CgroupsConfig;

const CGROUP_NAME: &str = "con";

pub fn run(config: &CgroupsConfig) -> Result<()> {
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
