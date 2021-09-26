use anyhow::Result;
use async_compression::tokio::bufread::GzipDecoder;
use clap::Clap;
use futures::future::join_all;
use oci_registry::registry::Registry;
use oci_spec::image::{Arch, Os};
use tokio::{
    fs::{create_dir_all, remove_file, rename, File},
    io::BufReader,
};
use tokio_tar::Archive;

use crate::image::{parse_image_id, Image, ImageId};

/// Pull an image or a repository from a registry
#[derive(Clap, Debug)]
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

                let mut tasks = vec![];

                for layer_path in image.layer_paths() {
                    tasks.push(tokio::spawn(async move {
                        let tar_gz = File::open(&layer_path).await?;
                        let tar = GzipDecoder::new(BufReader::new(tar_gz));
                        let mut archive = Archive::new(tar);

                        let digest = layer_path.file_name().unwrap().to_str().unwrap();
                        let unpacked_path = layer_path
                            .parent()
                            .unwrap()
                            .join(format!("{}-unpacked", digest));
                        create_dir_all(&unpacked_path).await?;
                        archive.unpack(&unpacked_path).await?;

                        remove_file(&layer_path).await?;
                        rename(
                            &unpacked_path,
                            &unpacked_path.parent().unwrap().join(digest),
                        )
                        .await?;

                        anyhow::Result::<()>::Ok(())
                    }));
                }
                join_all(tasks).await;

                Ok::<(), anyhow::Error>(())
            })?;

        Ok(())
    }
}
