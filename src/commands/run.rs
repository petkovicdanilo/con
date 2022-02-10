use std::{ffi::CString, fs::create_dir, path::Path, str::FromStr};

use crate::{
    container::{
        bundle::Bundle,
        capabilities,
        cgroups::{self, CGroup},
        env::EnvVariable,
        namespaces,
    },
    image::{parse_image_id, Image, ImageId},
    volume::Volume,
};
use anyhow::{anyhow, bail, Result};
use clap::Parser;
use nix::{
    sched::{clone, CloneFlags},
    sys::wait::{waitpid, WaitPidFlag},
    unistd::{self, execve, getpid},
};

use super::pull::Pull;

/// Run a command in a new container
#[derive(Parser, Debug)]
#[clap(author, version)]
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
    pub fn exec(mut self) -> Result<()> {
        let curr_dir = std::env::current_dir()?;
        let base_path = Path::new(&curr_dir).join(&self.image_id.name);

        if !base_path.exists() {
            let pull = Pull {
                image_id: self.image_id.clone(),
            };

            pull.exec()?;
        }

        if !base_path.is_dir() {
            bail!("Image directory not found");
        }

        let image = Image::new(self.image_id.name, self.image_id.tag, base_path)?;

        if let Some(config) = image.configuration.config() {
            if let Some(volumes) = config.volumes() {
                let config_volumes = volumes
                    .iter()
                    .map(|volume| -> Result<Volume> {
                        match Volume::from_str(volume) {
                            Ok(volume) => Ok(volume),
                            Err(err) => Err(anyhow!(err)),
                        }
                    })
                    .collect::<Result<Vec<Volume>>>()?;

                self.volumes.extend(config_volumes);
            }

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
        let cgroups_config = self.cgroups_config;

        namespaces::run(|| {
            let container_dir = std::env::current_dir()?.join(format!("{}-container", &image.name));
            create_dir(&container_dir)?;

            let bundle = Bundle::new(image.clone(), container_dir)?;

            bundle.mount_overlayfs()?;
            bundle.mount_volumes(volumes.iter())?;
            bundle.mount_special()?;

            unistd::sethostname(&hostname)?;

            capabilities::run()?;

            let mut cgroup = CGroup::new(&hostname, &cgroups_config)?;

            let child = Box::new(|| {
                let pid = getpid().as_raw() as u64;
                cgroup
                    .add_process(pid)
                    .expect("Failed adding process to cgroup");

                bundle
                    .change_root()
                    .expect("Failed setting container root file system");

                execve(
                    CString::new(command[0].clone()).unwrap().as_c_str(),
                    command[1..]
                        .into_iter()
                        .map(|c| CString::new(c.to_owned()).unwrap().as_c_str().to_owned())
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &env.iter()
                        .map(|e| {
                            CString::new(format!("{}={}", e.key, e.value))
                                .unwrap()
                                .as_c_str()
                                .to_owned()
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
                .expect("Error executing command");

                return 0;
            });

            let child_pid = clone(
                child,
                &mut [0u8; 1024 * 1024],
                CloneFlags::CLONE_NEWNS,
                None,
            )?;
            waitpid(child_pid, Some(WaitPidFlag::__WALL))?;

            cgroup.delete()?;

            bundle.unmount_special()?;
            bundle.unmount_volumes(volumes.iter())?;
            bundle.unmount_overlayfs()?;

            Ok::<(), anyhow::Error>(())
        })?;

        Ok(())
    }
}
