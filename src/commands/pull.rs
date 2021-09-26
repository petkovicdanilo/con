use anyhow::Result;
use clap::Clap;
use flate2::read::GzDecoder;
use oci_registry::registry::Registry;
use oci_spec::image::{Arch, Os};
use tar::Archive;
use tokio::fs::{create_dir_all, remove_file, rename, File};

use crate::image::{parse_image_id, Image, ImageId};

#[derive(Clap, Debug)]
#[clap(author, about, version)]
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

                let mut registry = Registry::new("https://registry-1.docker.io");
                registry
                    .pull_image(
                        &self.image_id.name,
                        &self.image_id.tag,
                        &Os::Linux,
                        &Arch::Amd64,
                        &destination_dir,
                    )
                    .await?;

                let image = Image::new(self.image_id.name, self.image_id.tag, destination_dir)?;

                for layer_path in image.layer_paths() {
                    let tar_gz = File::open(&layer_path).await?;
                    let tar = GzDecoder::new(tar_gz.into_std().await);
                    let mut archive = Archive::new(tar);

                    let digest = layer_path.file_name().unwrap().to_str().unwrap();
                    let unpacked_path = layer_path
                        .parent()
                        .unwrap()
                        .join(format!("{}-unpacked", digest));
                    archive.unpack(&unpacked_path)?;

                    remove_file(&layer_path).await?;
                    rename(
                        &unpacked_path,
                        &unpacked_path.parent().unwrap().join(digest),
                    )
                    .await?;
                }

                Ok::<(), anyhow::Error>(())
            })?;

        Ok(())
    }
}
