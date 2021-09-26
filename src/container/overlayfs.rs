use std::{
    fs::{create_dir, remove_dir_all},
    path::PathBuf,
};

use anyhow::Result;
use nix::mount::{mount, umount, MsFlags};

use crate::image::Image;

pub struct Bundle<'a> {
    pub(crate) dir: &'a PathBuf,
}

impl<'a> Bundle<'a> {
    pub fn new(image: &'a Image, dir: &'a PathBuf) -> Result<Self> {
        let root_path = Self::root_path_inner(&dir);
        create_dir(&root_path)?;

        let workdir_path = Self::workdir_path_inner(&dir);
        create_dir(&workdir_path)?;

        let upperdir_path = Self::upperdir_path_inner(&dir);
        create_dir(&upperdir_path)?;

        let layer_paths: Vec<String> = image
            .layer_paths()
            .iter()
            .map(|p| p.to_str().unwrap().to_string())
            .collect();

        mount(
            None::<&str>,
            &root_path,
            Some("overlay"),
            MsFlags::empty(),
            Some(
                format!(
                    "lowerdir={},upperdir={},workdir={}",
                    layer_paths.join(":"),
                    &upperdir_path.to_str().unwrap(),
                    &workdir_path.to_str().unwrap()
                )
                .as_str(),
            ),
        )?;

        Ok(Self { dir })
    }

    fn root_path_inner(dir: &'a PathBuf) -> PathBuf {
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
}

impl<'a> Drop for Bundle<'a> {
    fn drop(&mut self) {
        println!("Removing bundle...");
        umount(self.root_path().as_path()).unwrap();
        remove_dir_all(self.dir).unwrap();
    }
}
