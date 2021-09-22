use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::{
    container::{namespaces, overlayfs::Bundle},
    image::Image,
};
use anyhow::{bail, Result};
use clap::Clap;
use nix::unistd;
use tokio::fs::create_dir;

use super::pull::Pull;

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
    pub async fn exec(self) -> Result<()> {
        let base_path = Path::new(&std::env::current_dir()?).join(&self.image);

        if !base_path.exists() {
            let pull = Pull {
                image: self.image.clone(),
                tag: self.tag.clone(),
            };

            pull.exec().await?;
        }

        if !base_path.is_dir() {
            bail!("Image directory not found");
        }

        let image = Image::new(self.image, self.tag, base_path).await?;

        let container_dir = &std::env::current_dir()?.join(format!("{}-container", &image.name));
        create_dir(&container_dir).await?;
        let bundle = Bundle::new(&image, &container_dir)?;

        let name = self.name;

        namespaces::run(Box::new(|| {
            unistd::sethostname(&name).unwrap();

            println!("{}", bundle.root_path().to_str().unwrap());

            let mut c = Command::new("/bin/bash")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("failed to start");

            c.wait().expect("error");

            return 0;
        }))?;

        Ok(())
    }
}
