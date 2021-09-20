use std::path::PathBuf;

pub fn split_digest<'a>(digest: &'a str) -> (&'a str, &'a str) {
    digest.split_once(":").unwrap()
}

pub fn blob_path(base_path: &PathBuf, digest: &str) -> PathBuf {
    let (alg, digest) = split_digest(digest);
    base_path.join(format!("blobs/{}/{}", alg, digest))
}
