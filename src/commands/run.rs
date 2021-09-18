use anyhow::{bail, Result};
use clap::Clap;
use flate2::read::GzDecoder;
use nix::mount::{mount, umount, MsFlags};
use oci_spec::image::{ImageIndex, ImageManifest};
use std::{
    fs::{create_dir, File},
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
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

        let layer_paths = manifest
            .layers()
            .iter()
            .map(|layer| -> Result<String> {
                let digest = layer.digest();

                let tar_gz = File::open(blob_path(&image_path, &digest))?;
                let tar = GzDecoder::new(tar_gz);
                let mut archive = Archive::new(tar);

                let (_, digest) = split_digest(digest);
                let path = container_dir.join(digest);
                archive.unpack(path.clone())?;

                Ok(path.to_str().unwrap().to_string())
            })
            .collect::<Result<Vec<_>>>()?;

        let root_path = container_dir.join("rootfs");
        create_dir(root_path.clone())?;

        let workdir_path = container_dir.join("workdir");
        create_dir(workdir_path.clone())?;

        let upperdir_path = container_dir.join("upperdir");
        create_dir(upperdir_path.clone())?;

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

fn split_digest<'a>(digest: &'a str) -> (&'a str, &'a str) {
    digest.split_once(":").unwrap()
}

fn blob_path(base_path: &PathBuf, digest: &str) -> PathBuf {
    let (alg, digest) = split_digest(digest);
    base_path.join(format!("blobs/{}/{}", alg, digest))
}
