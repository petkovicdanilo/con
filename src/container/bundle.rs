use std::{
    fs::{create_dir, remove_dir, remove_dir_all},
    path::PathBuf,
};

use anyhow::Result;
use nix::{
    mount::{mount, umount, umount2, MntFlags, MsFlags},
    unistd::{chdir, pivot_root},
};

use crate::{image::Image, volume::Volume};

pub struct Bundle {
    pub(crate) dir: PathBuf,
    pub(crate) image: Image,
}

impl Bundle {
    pub fn new(image: Image, dir: PathBuf) -> Result<Self> {
        let root_path = Self::root_path_inner(&dir);
        create_dir(&root_path)?;

        let workdir_path = Self::workdir_path_inner(&dir);
        create_dir(&workdir_path)?;

        let upperdir_path = Self::upperdir_path_inner(&dir);
        create_dir(&upperdir_path)?;

        Ok(Self { image, dir })
    }

    fn root_path_inner(dir: &PathBuf) -> PathBuf {
        dir.join("rootfs")
    }

    pub fn root_path(&self) -> PathBuf {
        Self::root_path_inner(&self.dir)
    }

    fn workdir_path_inner(dir: &PathBuf) -> PathBuf {
        dir.join("workdir")
    }

    pub fn workdir_path(&self) -> PathBuf {
        Self::workdir_path_inner(&self.dir)
    }

    fn upperdir_path_inner(dir: &PathBuf) -> PathBuf {
        dir.join("upperdir")
    }

    pub fn upperdir_path(&self) -> PathBuf {
        Self::upperdir_path_inner(&self.dir)
    }

    pub fn host_path_from_container_path(&self, inner_path: &PathBuf) -> Result<PathBuf> {
        let path = if inner_path.starts_with("/") {
            inner_path.strip_prefix("/")?
        } else {
            inner_path
        };

        Ok(self.root_path().join(&path))
    }

    pub fn mount_overlayfs(&self) -> Result<()> {
        let layer_paths: Vec<String> = self
            .image
            .layer_paths()
            .iter()
            .map(|p| p.to_str().unwrap().to_string())
            .collect();

        mount(
            None::<&str>,
            &self.root_path(),
            Some("overlay"),
            MsFlags::empty(),
            Some(
                format!(
                    "lowerdir={},upperdir={},workdir={}",
                    layer_paths.join(":"),
                    &self.upperdir_path().to_str().unwrap(),
                    &self.workdir_path().to_str().unwrap()
                )
                .as_str(),
            ),
        )?;

        Ok(())
    }

    pub fn unmount_overlayfs(&self) -> Result<()> {
        umount(&self.root_path())?;
        remove_dir_all(&self.dir)?;

        Ok(())
    }

    pub fn change_root(&self) -> Result<()> {
        let root_path = self.root_path();
        let old_root = root_path.join("old_root");
        create_dir(&old_root)?;

        mount(
            None::<&str>,
            "/",
            None::<&str>,
            MsFlags::MS_REC | MsFlags::MS_PRIVATE,
            None::<&str>,
        )?;

        pivot_root(&root_path, &old_root)?;
        chdir("/")?;

        umount2("/old_root", MntFlags::MNT_DETACH)?;
        remove_dir("/old_root")?;

        Ok(())
    }

    pub fn mount_special(&self) -> Result<()> {
        let root_path = self.root_path();

        let oldproc = root_path.join(".oldproc");
        create_dir(&oldproc)?;
        mount(
            Some("/proc"),
            &oldproc,
            None::<&str>,
            MsFlags::MS_REC | MsFlags::MS_BIND,
            None::<&str>,
        )?;

        let proc_path = root_path.join("proc");
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

        umount2(&oldproc, MntFlags::MNT_DETACH)?;
        remove_dir(&oldproc)?;

        let tmp_path = root_path.join("tmp");
        if !tmp_path.exists() {
            create_dir(&tmp_path)?;
        }

        mount(
            Some("tmp"),
            &root_path.join("tmp"),
            Some("tmpfs"),
            MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_NOATIME,
            None::<&str>,
        )?;

        let sys = root_path.join("sys");
        mount(
            Some("/sys"),
            &sys,
            None::<&str>,
            MsFlags::MS_REC | MsFlags::MS_BIND,
            None::<&str>,
        )?;

        Ok(())
    }

    pub fn unmount_special(&self) -> Result<()> {
        let root_path = self.root_path();

        umount(&root_path.join("proc"))?;
        umount(&root_path.join("tmp"))?;
        umount2(&root_path.join("sys"), MntFlags::MNT_DETACH)?;

        Ok(())
    }

    pub fn mount_volumes<'a, I>(&self, volumes: I) -> Result<()>
    where
        I: Iterator<Item = &'a Volume>,
    {
        for volume in volumes {
            let destination_full_path = self.host_path_from_container_path(&volume.destination)?;

            if !destination_full_path.exists() {
                create_dir(&destination_full_path)?;
            }

            mount(
                Some(&volume.source),
                &destination_full_path,
                None::<&str>,
                MsFlags::MS_BIND,
                None::<&str>,
            )?;
        }

        Ok(())
    }

    pub fn unmount_volumes<'a, I>(&self, volumes: I) -> Result<()>
    where
        I: Iterator<Item = &'a Volume>,
    {
        for volume in volumes {
            umount(&self.host_path_from_container_path(&volume.destination)?)?;
        }

        Ok(())
    }
}
