use anyhow::Result;
use nix::{
    libc::size_t,
    sched::{self, CloneCb, CloneFlags},
    sys::wait,
};

pub fn run(callback: CloneCb) -> Result<()> {
    const STACK_SIZE: size_t = 1024 * 1024;
    let mut stack = [0u8; STACK_SIZE];

    let flags = CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWCGROUP
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWIPC
        | CloneFlags::CLONE_NEWNET
        | CloneFlags::CLONE_NEWUTS;

    let child_pid = sched::clone(callback, &mut stack, flags, None)?;
    wait::waitpid(child_pid, Some(wait::WaitPidFlag::__WCLONE))?;

    Ok(())
}
