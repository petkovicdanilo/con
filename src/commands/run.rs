use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::{
    container::{capabilities, cgroups, mounts, namespaces, overlayfs::Bundle},
    image::{parse_image_id, Image, ImageId},
};
use anyhow::{bail, Result};
use clap::Clap;
use nix::unistd;
use tokio::fs::create_dir;

use super::pull::Pull;

#[derive(Clap, Debug)]
pub struct Run {
    /// Container host name
    #[clap(long, default_value = "container")]
    hostname: String,

    #[clap(flatten)]
    cgroups_config: CgroupsConfig,


    #[clap(name = "IMAGE", parse(from_str = parse_image_id))]
    image_id: ImageId,

    command: Vec<String>,
}

#[derive(Clap, Debug)]
pub struct CgroupsConfig {
    /// CPU shares (relative weight)
    #[clap(short, long, default_value = "256")]
    pub(crate) cpu_shares: u64,

    /// Memory limit in bytes
    #[clap(short, long, default_value = "1073741824")]
    pub(crate) memory: u64,

    /// Tune container pids limit (0 for unlimited)
    #[clap(short, long, default_value = "0")]
    pub(crate) pids_limit: u32,
}

impl Run {
    pub async fn exec(self) -> Result<()> {
        let base_path = Path::new(&std::env::current_dir()?).join(&self.image_id.name);

        if !base_path.exists() {
            let pull = Pull {
                image_id: self.image_id.clone(),
            };

            pull.exec().await?;
        }

        if !base_path.is_dir() {
            bail!("Image directory not found");
        }

        let image = Image::new(self.image_id.name, self.image_id.tag, base_path).await?;

        let container_dir = &std::env::current_dir()?.join(format!("{}-container", &image.name));
        create_dir(&container_dir).await?;
        let bundle = Bundle::new(&image, &container_dir)?;

        let hostname = self.hostname;
        let command = self.command;

        cgroups::run(&self.cgroups_config)?;

        namespaces::run(Box::new(|| {
            unistd::sethostname(&hostname).unwrap();

            mounts::change_root(&bundle).unwrap();
            mounts::special_mount().unwrap();

            capabilities::run().unwrap();

            unsafe {
                nix::env::clearenv().unwrap();
            };

            if let Some(config) = image.configuration.config() {
                if let Some(env) = config.env() {
                    for var in env {
                        let (key, value) = var.split_once("=").unwrap();
                        std::env::set_var(key, value);
                    }
                }
            }

            let mut c = Command::new(command[0].as_str())
                .args(command[1..].as_ref())
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("failed to start");

            c.wait().expect("error");

            mounts::special_unmount().unwrap();

            return 0;
        }))?;

        Ok(())
    }
}
