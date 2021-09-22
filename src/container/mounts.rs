use std::fs::{create_dir, remove_dir};

use anyhow::Result;
use nix::{
    mount::{mount, umount, umount2, MntFlags, MsFlags},
    unistd::{chdir, pivot_root},
};

use super::overlayfs::Bundle;

pub fn change_root(bundle: &Bundle) -> Result<()> {
    let old_root = bundle.root_path().join("old_root");
    create_dir(&old_root)?;

    mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_REC | MsFlags::MS_PRIVATE,
        None::<&str>,
    )?;

    pivot_root(&bundle.root_path(), &old_root)?;
    chdir("/")?;

    umount2("/old_root", MntFlags::MNT_DETACH)?;
    remove_dir("/old_root")?;

    Ok(())
}

pub fn special_mount() -> Result<()> {
    mount(
        Some("proc"),
        "/proc",
        Some("proc"),
        MsFlags::empty(),
        None::<&str>,
    )?;

    mount(
        Some("tmp"),
        "/tmp",
        Some("tmpfs"),
        MsFlags::empty(),
        None::<&str>,
    )?;

    Ok(())
}

pub fn special_unmount() -> Result<()> {
    umount("/proc")?;
    umount("/tmp")?;

    Ok(())
}
