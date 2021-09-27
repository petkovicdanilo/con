use std::fs::{create_dir, remove_dir};
use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use nix::{
    mount::{mount, umount, umount2, MntFlags, MsFlags},
    unistd::{chdir, pivot_root},
};

use super::overlayfs::Bundle;

#[derive(Debug)]
pub struct Volume {
    pub source: PathBuf,
    pub destination: PathBuf,
}

impl FromStr for Volume {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(":") {
            Some((source, destination)) => {
                let source = match PathBuf::from_str(source) {
                    Ok(source) => source,
                    Err(_) => return Err("Source is not a path"),
                };

                let destination = match PathBuf::from_str(destination) {
                    Ok(destination) => destination,
                    Err(_) => return Err("Destination is not a path"),
                };

                Ok(Self {
                    source,
                    destination,
                })
            }
            None => Err("Invalid volume syntax. Expected in format 'source:destination'"),
        }
    }
}

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

pub fn mount_special(bundle: &Bundle) -> Result<()> {
    let proc_path = bundle.root_path().join("proc");
    if !proc_path.exists() {
        create_dir(&proc_path)?;
    }

    mount(
        Some("proc"),
        &proc_path,
        Some("proc"),
        MsFlags::MS_NOSUID,
        None::<&str>,
    )?;

    let oldproc_path = bundle.root_path().join(".oldproc");
    umount2(&oldproc_path, MntFlags::MNT_DETACH)?;
    remove_dir(&oldproc_path)?;

    let tmp_path = bundle.root_path().join("tmp");
    if !tmp_path.exists() {
        create_dir(&tmp_path)?;
    }

    mount(
        Some("tmp"),
        &bundle.root_path().join("tmp"),
        Some("tmpfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_NOATIME,
        None::<&str>,
    )?;

    Ok(())
}

pub fn unmount_special(bundle: &Bundle) -> Result<()> {
    umount(&bundle.root_path().join("proc"))?;
    umount(&bundle.root_path().join("tmp"))?;
    umount2(&bundle.root_path().join("sys"), MntFlags::MNT_DETACH)?;

    Ok(())
}
