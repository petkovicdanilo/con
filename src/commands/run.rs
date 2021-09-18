use anyhow::{bail, Result};
use clap::Clap;
use flate2::read::GzDecoder;
use oci_spec::image::{ImageIndex, ImageManifest};
use std::{
    fs::{create_dir, File},
    path::{Path, PathBuf},
};
use tar::Archive;

#[derive(Clap, Debug)]
pub struct Run {
    image: String,

    command: Vec<String>,
}

impl Run {
    pub fn exec(&self) -> Result<()> {
        let image_path = Path::new(&std::env::current_dir()?).join(&self.image);

        if !image_path.exists() || !image_path.is_dir() {
            bail!("Image directory not found");
        }

        let index = ImageIndex::from_file(image_path.join("index.json"))?;

        let manifest_digest = &index.manifests()[0].digest();
        let manifest = ImageManifest::from_file(blob_path(&image_path, &manifest_digest))?;

        let container_dir = &std::env::current_dir()?.join(format!("{}-container", &self.image));
        create_dir(container_dir)?;

        for layer in manifest.layers() {
            let digest = layer.digest();

            let tar_gz = File::open(blob_path(&image_path, &digest))?;
            let tar = GzDecoder::new(tar_gz);
            let mut archive = Archive::new(tar);

            let (_, digest) = split_digest(digest);
            archive.unpack(container_dir.join(digest))?;
        }

        Ok(())
    }
}

fn split_digest<'a>(digest: &'a str) -> (&'a str, &'a str) {
    digest.split_once(":").unwrap()
}

fn blob_path(base_path: &PathBuf, digest: &str) -> PathBuf {
    let (alg, digest) = split_digest(digest);
    base_path.join(format!("blobs/{}/{}", alg, digest))
}
