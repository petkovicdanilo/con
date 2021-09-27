use std::{
    fs::{create_dir, remove_dir_all},
    path::PathBuf,
};

use anyhow::Result;
use nix::mount::{mount, umount, MsFlags};

use crate::image::Image;

use super::mounts::Volume;

pub struct Bundle {
    pub(crate) dir: PathBuf,
    pub(crate) image: Image,
    pub(crate) volumes: Vec<Volume>,
}

impl Bundle {
    pub fn new(image: Image, dir: PathBuf) -> Result<Self> {
        let root_path = Self::root_path_inner(&dir);
        create_dir(&root_path)?;

        let workdir_path = Self::workdir_path_inner(&dir);
        create_dir(&workdir_path)?;

        let upperdir_path = Self::upperdir_path_inner(&dir);
        create_dir(&upperdir_path)?;

        Ok(Self {
            image,
            dir,
            volumes: Vec::new(),
        })
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
        umount(self.root_path().as_path())?;
        remove_dir_all(&self.dir)?;

        Ok(())
    }

    pub fn mount_volumes<I>(&mut self, volumes: I) -> Result<()>
    where
        I: Iterator<Item = Volume>,
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

            self.volumes.push(volume);
        }

        Ok(())
    }

    pub fn unmount_volumes(&mut self) -> Result<()> {
        for volume in &self.volumes {
            let destination_full_path = self.host_path_from_container_path(&volume.destination)?;

            umount(&destination_full_path)?;
        }

        self.volumes.clear();

        Ok(())
    }
}
