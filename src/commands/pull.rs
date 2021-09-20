use anyhow::Result;
use clap::Clap;
use flate2::read::GzDecoder;
use oci_registry::registry::Registry;
use oci_spec::image::{Arch, Os};
use tar::Archive;
use tokio::fs::{create_dir_all, remove_file, rename, File};

use crate::image::Image;

#[derive(Clap, Debug)]
pub struct Pull {
    pub image: String,

    #[clap(default_value = "latest")]
    pub tag: String,
}

impl Pull {
    pub async fn exec(self) -> Result<()> {
        let destination_dir = std::env::current_dir()?.join(&self.image);
        create_dir_all(&destination_dir).await?;

        let mut registry = Registry::new("https://registry-1.docker.io");
        registry
            .pull_image(
                &self.image,
                &self.tag,
                &Os::Linux,
                &Arch::Amd64,
                &destination_dir,
            )
            .await?;

        let image = Image::new(self.image, self.tag, destination_dir).await?;

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

        Ok(())
    }
}
