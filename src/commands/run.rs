use anyhow::{bail, Result};
use clap::Clap;
use nix::mount::{mount, umount, MsFlags};
use std::{path::Path, thread::sleep, time::Duration};
use tokio::fs::create_dir;

use crate::{commands::pull::Pull, image::Image};

#[derive(Clap, Debug)]
pub struct Run {
    #[clap(long, default_value = "container")]
    name: String,

    image: String,

    #[clap(default_value = "latest")]
    tag: String,

    command: Vec<String>,
}

impl Run {
    pub async fn exec(&self) -> Result<()> {
        let image_path = Path::new(&std::env::current_dir()?).join(&self.image);

        if !image_path.exists() {
            let pull = Pull {
                image: self.image.clone(),
                tag: self.tag.clone(),
            };

            pull.exec().await?;
        }

        if !image_path.is_dir() {
            bail!("Image directory not found");
        }

        let image = Image::from_path(image_path)?;

        let container_dir = &std::env::current_dir()?.join(format!("{}-container", &self.image));
        create_dir(container_dir).await?;

        let root_path = container_dir.join("rootfs");
        create_dir(root_path.clone()).await?;

        let workdir_path = container_dir.join("workdir");
        create_dir(workdir_path.clone()).await?;

        let upperdir_path = container_dir.join("upperdir");
        create_dir(upperdir_path.clone()).await?;

        let layer_paths: Vec<String> = image
            .layer_paths()
            .iter()
            .map(|p| p.to_str().unwrap().to_string())
            .collect();

        mount(
            None::<&str>,
            root_path.as_path(),
            Some("overlay"),
            MsFlags::empty(),
            Some(
                format!(
                    "lowerdir={},upperdir={},workdir={}",
                    layer_paths.join(":"),
                    upperdir_path.to_str().unwrap(),
                    workdir_path.to_str().unwrap()
                )
                .as_str(),
            ),
        )?;

        sleep(Duration::from_secs(20));

        umount(root_path.as_path())?;

        Ok(())
    }
}
