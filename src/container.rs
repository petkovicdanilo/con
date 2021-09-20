use std::{fs::remove_dir_all, path::PathBuf, time::Duration};

use crate::image::Image;
use anyhow::Result;
use nix::mount::{mount, umount, MsFlags};
use std::thread::sleep;
use tokio::fs::create_dir;

pub struct Container {
    pub image: Image,
    dir: PathBuf,
}

impl Container {
    pub async fn new(image: Image, dir: PathBuf) -> Result<Self> {
        let root_path = root_path(&dir);
        create_dir(&root_path).await?;

        let workdir_path = workdir_path(&dir);
        create_dir(&workdir_path).await?;

        let upperdir_path = upperdir_path(&dir);
        create_dir(&upperdir_path).await?;

        Ok(Self { dir, image })
    }

    pub async fn run(&self, _command: Vec<String>) -> Result<()> {
        let layer_paths: Vec<String> = self
            .image
            .layer_paths()
            .iter()
            .map(|p| p.to_str().unwrap().to_string())
            .collect();

        mount(
            None::<&str>,
            root_path(&self.dir).as_path(),
            Some("overlay"),
            MsFlags::empty(),
            Some(
                format!(
                    "lowerdir={},upperdir={},workdir={}",
                    layer_paths.join(":"),
                    upperdir_path(&self.dir).to_str().unwrap(),
                    workdir_path(&self.dir).to_str().unwrap()
                )
                .as_str(),
            ),
        )?;

        sleep(Duration::from_secs(20));

        Ok(())
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        umount(root_path(&self.dir).as_path()).unwrap();
        remove_dir_all(&self.dir).unwrap();
    }
}

fn root_path(dir: &PathBuf) -> PathBuf {
    dir.join("rootfs")
}

fn workdir_path(dir: &PathBuf) -> PathBuf {
    dir.join("workdir")
}

fn upperdir_path(dir: &PathBuf) -> PathBuf {
    dir.join("upperdir")
}
