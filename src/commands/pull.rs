use std::{fs::File, io::BufReader};

use anyhow::Result;
use clap::Parser;
use flate2::bufread::GzDecoder;
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use oci_registry::Registry;
use oci_spec::image::{Arch, Os};
use tar::Archive;
use tokio::fs::{create_dir_all, remove_file, rename};

use crate::image::{parse_image_id, Image, ImageId};

/// Pull an image or a repository from a registry
#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct Pull {
    #[clap(name = "IMAGE", parse(from_str = parse_image_id))]
    pub image_id: ImageId,
}

impl Pull {
    pub fn exec(self) -> Result<()> {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let destination_dir = std::env::current_dir()?.join(&self.image_id.name);
                create_dir_all(&destination_dir).await?;

                let registry = Registry::new("https://registry-1.docker.io");
                registry
                    .pull_image_with_progress_bar(
                        &self.image_id.name,
                        &self.image_id.tag,
                        &Os::Linux,
                        &Arch::Amd64,
                        &destination_dir,
                    )
                    .await?;

                let image = Image::new(self.image_id.name, self.image_id.tag, destination_dir)?;

                let multi_progress = MultiProgress::new();
                let mut tasks = vec![];

                for layer_path in image.layer_paths() {
                    let progress_bar = multi_progress.add(ProgressBar::new(0));

                    tasks.push(tokio::spawn(async move {
                        let tar_gz = File::open(&layer_path)?;
                        let buf_reader = BufReader::new(tar_gz);
                        let tar = GzDecoder::new(buf_reader);
                        let mut archive = Archive::new(tar);

                        let digest = layer_path.file_name().unwrap().to_str().unwrap();
                        let alg = layer_path
                            .parent()
                            .unwrap()
                            .components()
                            .last()
                            .unwrap()
                            .as_os_str()
                            .to_str()
                            .unwrap();
                        let unpacked_path = layer_path
                            .parent()
                            .unwrap()
                            .join(format!("{}-unpacked", digest));

                        progress_bar.set_style(ProgressStyle::default_bar().template("{msg}"));
                        progress_bar.set_message(format!("[ ] Unpacking {}:{}", alg, digest));

                        create_dir_all(&unpacked_path).await?;
                        archive.unpack(&unpacked_path)?;

                        remove_file(&layer_path).await?;
                        rename(
                            &unpacked_path,
                            &unpacked_path.parent().unwrap().join(digest),
                        )
                        .await?;

                        progress_bar
                            .finish_with_message(format!("[x] Unpacked  {}:{}", alg, digest));
                        anyhow::Result::<()>::Ok(())
                    }));
                }

                let handle_m = tokio::task::spawn_blocking(move || multi_progress.join().unwrap());
                join_all(tasks).await;
                handle_m.await.unwrap();

                Ok::<(), anyhow::Error>(())
            })?;

        Ok(())
    }
}
