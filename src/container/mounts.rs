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

pub fn mount_special() -> Result<()> {
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

pub fn unmount_special() -> Result<()> {
    umount("/proc")?;
    umount("/tmp")?;

    Ok(())
}
