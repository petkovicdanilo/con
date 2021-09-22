use std::fs::{create_dir, remove_dir};

use anyhow::Result;
use nix::{
    mount::{mount, umount2, MntFlags, MsFlags},
    unistd::{chdir, pivot_root},
};

use super::overlayfs::Bundle;

pub fn run(bundle: &Bundle) -> Result<()> {
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
