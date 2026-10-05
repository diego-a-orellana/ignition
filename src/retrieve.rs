// Asset retrieval and extraction for build.

use std::{
    ffi::OsString,
    fs::{File, create_dir_all, rename},
    io::copy,
    path::{Path, PathBuf},
};

use flate2::read::GzDecoder;
use tar::Archive;

use crate::asset::*;
use crate::config::*;
use crate::consts::*;
use crate::error::*;

/// Paths as remote source and local destinations of a single asset.
///
/// Assets are organized, remotely and in the cache, by asset key and target triplet:
/// `<asset-directory>/<asset>/<target-triplet>/<asset>.tar.gz`.
struct Paths {
    /// Url of the remote asset archive.
    url: String,
    /// Cached path of the retrieved asset archive.
    archive: PathBuf,
    /// Directory the asset archive is extracted into.
    extract: PathBuf,
}

impl Paths {
    /// Locate an asset for the configured target.
    fn new<T: Retrievable>(config: &Config) -> Self {
        let relative_asset_path = Path::new(&config.asset_dir)
            .join(T::KEY)
            .join(&config.target)
            .join(format!("{}{}", T::KEY, ARCHIVE_EXTENSION));
        Self {
            url: format!(
                "{}/{}",
                config.bucket_url.trim_end_matches('/'),
                relative_asset_path.to_string_lossy()
            ),
            archive: Path::new(&config.build_dir)
                .join(&config.cache_dir)
                .join(&relative_asset_path),
            extract: config.asset_path().join(T::KEY),
        }
    }
}

/// Retrieve and extract an asset for the configured target, exporting the archive path
/// as cargo metadata so that dependent crates may verify it.
pub fn asset_retrieve<T: Retrievable>(_: &T, config: &Config) -> IgnitionResult<PathBuf> {
    let paths = Paths::new::<T>(config);
    if let Some(cache_dir) = paths.archive.parent() {
        directory(cache_dir)?;
    }
    directory(&paths.extract)?;

    // A cached archive is complete by construction, so retrieval is skipped
    if !paths.archive.exists() {
        let size = validate(&paths.url)?;
        download(&paths.url, &paths.archive, size)?;
    }
    extract(&paths.archive, &paths.extract)?;

    println!(
        "cargo::metadata={}{}={}",
        T::KEY.to_uppercase(),
        METADATA_PATH_KEY_SUFFIX,
        paths.archive.to_string_lossy()
    );
    Ok(paths.archive)
}

/// Create a directory, and any missing parent, if not already present.
fn directory(directory: &Path) -> IgnitionResult<()> {
    create_dir_all(directory)
        .map_err(|err| IgnitionError::FileSystemError(directory.to_path_buf(), err.to_string()))
}

/// Download an asset archive, renaming it into place once complete.
fn download(url: &str, archive: &Path, size: u64) -> IgnitionResult<()> {
    let mut partial = OsString::from(archive);
    partial.push(ARCHIVE_PARTIAL_EXTENSION);
    let partial = PathBuf::from(partial);

    let response = ureq::get(url).call().map_err(|err| request(url, err))?;
    let mut file = File::create(&partial)
        .map_err(|err| IgnitionError::FileSystemError(partial.clone(), err.to_string()))?;
    let written = copy(&mut response.into_reader(), &mut file)
        .map_err(|err| IgnitionError::FileSystemError(partial.clone(), err.to_string()))?;

    // Only a complete download is renamed into place, so that a partial archive is never cached
    if written != size {
        return Err(IgnitionError::DownloadError(url.to_string(), size, written));
    }
    rename(&partial, archive)
        .map_err(|err| IgnitionError::FileSystemError(partial, err.to_string()))
}

/// Extract an asset archive into its destination directory.
fn extract(archive: &Path, directory: &Path) -> IgnitionResult<()> {
    let file = File::open(archive)
        .map_err(|err| IgnitionError::FileSystemError(archive.to_path_buf(), err.to_string()))?;
    Archive::new(GzDecoder::new(file))
        .unpack(directory)
        .map_err(|err| IgnitionError::ArchiveError(archive.to_path_buf(), err.to_string()))
}

/// Describe a failed request, whose url is reported by the error itself.
fn request(url: &str, err: ureq::Error) -> IgnitionError {
    let detail = match err {
        ureq::Error::Status(code, _) => format!("status code {}", code),
        ureq::Error::Transport(transport) => transport.to_string(),
    };
    IgnitionError::RequestError(url.to_string(), detail)
}

/// Validate that a remote asset archive exists, returning its size in bytes.
fn validate(url: &str) -> IgnitionResult<u64> {
    let response = ureq::head(url).call().map_err(|err| request(url, err))?;
    response
        .header("content-length")
        .and_then(|size| size.parse().ok())
        .ok_or(IgnitionError::RequestError(
            url.to_string(),
            "missing or invalid content-length".to_string(),
        ))
}
