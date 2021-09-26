use std::{
    path::Path,
    process::{Command, Stdio},
    str::FromStr,
};

use crate::{
    container::{
        capabilities, cgroups,
        env::{self, EnvVariable},
        mounts::{self, Volume},
        namespaces,
        overlayfs::Bundle,
    },
    image::{parse_image_id, Image, ImageId},
};
use anyhow::{anyhow, bail, Result};
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
    cgroups_config: cgroups::Config,

    /// Bind mount a volume
    #[clap(short, long, multiple_occurrences(true), number_of_values = 1)]
    volumes: Vec<Volume>,

    /// Set environment variables
    #[clap(short, long, multiple_occurrences(true), number_of_values = 1)]
    env: Vec<EnvVariable>,

    #[clap(name = "IMAGE", parse(from_str = parse_image_id))]
    image_id: ImageId,

    command: Vec<String>,
}

impl Run {
    pub async fn exec(mut self) -> Result<()> {
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

        if let Some(config) = image.configuration.config() {
            if let Some(env) = config.env() {
                let config_vars = env
                    .iter()
                    .map(|var| -> Result<EnvVariable> {
                        match EnvVariable::from_str(var) {
                            Ok(var) => Ok(var),
                            Err(err) => Err(anyhow!(err)),
                        }
                    })
                    .collect::<Result<Vec<EnvVariable>>>()?;

                self.env.extend(config_vars);
            }
        }

        let hostname = self.hostname;
        let command = self.command;
        let volumes = self.volumes;
        let env = self.env;

        cgroups::run(&self.cgroups_config)?;
        mounts::mount_volumes(volumes.iter(), &bundle).unwrap();

        namespaces::run(Box::new(|| {
            unistd::sethostname(&hostname).unwrap();

            mounts::change_root(&bundle).unwrap();
            mounts::mount_special().unwrap();

            capabilities::run().unwrap();

            unsafe {
                nix::env::clearenv().unwrap();
            };

            env::set_variables(env.iter()).unwrap();

            let mut c = Command::new(command[0].as_str())
                .args(command[1..].as_ref())
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("failed to start");

            c.wait().expect("error");

            mounts::unmount_special().unwrap();

            return 0;
        }))?;

        mounts::unmount_volumes(volumes.iter(), &bundle).unwrap();
        Ok(())
    }
}
