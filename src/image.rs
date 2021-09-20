use anyhow::Result;
use oci_spec::image::{ImageIndex, ImageManifest};
use std::path::PathBuf;

use crate::util::blob_path;

pub struct Image {
    pub base_path: PathBuf,
    pub index: ImageIndex,
    pub manifest: ImageManifest,
}

impl Image {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let index = ImageIndex::from_file(path.join("index.json"))?;

        let manifest_digest = &index.manifests()[0].digest();
        let manifest = ImageManifest::from_file(blob_path(&path, &manifest_digest))?;

        Ok(Self {
            base_path: path,
            index,
            manifest,
        })
    }

    pub fn layer_paths(&self) -> Vec<PathBuf> {
        self.manifest
            .layers()
            .iter()
            .map(|layer| blob_path(&self.base_path, layer.digest()))
            .collect()
    }
}
