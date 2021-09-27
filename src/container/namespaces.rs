use std::process::Command;

use anyhow::Result;
use nix::{
    libc::size_t,
    sched::{self, CloneFlags},
    sys::{
        socket::{socketpair, AddressFamily, SockFlag, SockType},
        wait,
    },
    unistd::{self, getgid, getuid, Pid},
};

pub fn run<F>(callback: F) -> Result<()>
where
    F: Fn() -> Result<()>,
{
    const STACK_SIZE: size_t = 1024 * 1024;
    let mut stack = [0u8; STACK_SIZE];

    let flags = CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWCGROUP
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWIPC
        | CloneFlags::CLONE_NEWNET
        | CloneFlags::CLONE_NEWUTS
        | CloneFlags::CLONE_NEWUSER;

    let (socket1, socket2) = socketpair(
        AddressFamily::Unix,
        SockType::SeqPacket,
        None,
        SockFlag::empty(),
    )?;

    let clone_callback = Box::new(|| {
        let mut buf = [0u8; 4];
        unistd::read(socket2, &mut buf).unwrap();

        if u32::from_le_bytes(buf) != 0 {
            panic!("Socket error");
        }

        match callback() {
            Ok(_) => return 0,
            Err(err) => panic!("Error: {}", err.to_string()),
        }
    });

    let child_pid = sched::clone(clone_callback, &mut stack, flags, None)?;

    configure_userns(&child_pid)?;
    unistd::write(socket1, &0_i32.to_le_bytes())?;

    wait::waitpid(child_pid, Some(wait::WaitPidFlag::__WCLONE))?;

    Ok(())
}

fn configure_userns(child_pid: &Pid) -> Result<()> {
    let uid = getuid().as_raw().to_string();
    Command::new("newuidmap")
        .args([&child_pid.to_string(), "0", &uid, "1"])
        .spawn()?
        .wait()?;

    let gid = getgid().as_raw().to_string();
    Command::new("newgidmap")
        .args([&child_pid.to_string(), "0", &gid, "1"])
        .spawn()?
        .wait()?;

    Ok(())
}
