use anyhow::Result;
use oci_spec::image::{ImageConfiguration, ImageIndex, ImageManifest};
use std::path::PathBuf;

use crate::util::blob_path;

#[derive(Clone, Debug)]
pub struct ImageId {
    pub name: String,
    pub tag: String,
}

pub fn parse_image_id(s: &str) -> ImageId {
    match s.split_once(":") {
        Some((name, tag)) => {
            let name = normalize_image_name(name);

            ImageId {
                name,
                tag: tag.to_string(),
            }
        }
        None => {
            let name = normalize_image_name(&s);

            ImageId {
                name,
                tag: String::from("latest"),
            }
        }
    }
}

fn normalize_image_name(name: &str) -> String {
    if name.contains("/") {
        name.to_string()
    } else {
        format!("library/{}", name)
    }
}

pub struct Image {
    pub name: String,
    pub tag: String,
    pub base_path: PathBuf,
    pub index: ImageIndex,
    pub manifest: ImageManifest,
    pub configuration: ImageConfiguration,
}

impl Image {
    pub fn new(name: String, tag: String, base_path: PathBuf) -> Result<Self> {
        let index = ImageIndex::from_file(base_path.join("index.json"))?;

        let manifest_digest = &index.manifests()[0].digest();
        let manifest = ImageManifest::from_file(blob_path(&base_path, &manifest_digest))?;

        let configuration_digest = &manifest.config().digest();
        let configuration =
            ImageConfiguration::from_file(blob_path(&base_path, &configuration_digest))?;

        Ok(Self {
            name,
            tag,
            base_path,
            index,
            manifest,
            configuration,
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
