use crate::{config, Error};
use fs_extra::dir::CopyOptions;
use std::path::Path;

const TARGET_FOLDER_NAME: &str = "_";

/// Retrieve dependency files based on a configuration file located at `config_path`.
///
/// Dependencies are put in a `_` folder next to the configuration file.
/// If the dependency already exists, then the dependency is not retrieved again.
///
/// # Configuration file format
///
/// The YAML configuration file has the following format:
/// ```yaml
/// dependencies:
///   dependency1_name:
///     # path is relative to configuration file folder
///     # dependency is retrieved from local path '../deps/dependency1_name' (symbolic link is created)
///     path: ../deps/
///   dependency2_name:
///     # dependency is located in '<first root folder>/dependency2_name' folder of ZIP file
///     # accessible by URL
///     url: https://github.com/orga/project/archive/refs/heads/main.zip
/// ```
///
/// # Errors
///
/// An error is returned if the configuration file is invalid or if there is an issue during
/// dependency retrieval.
pub fn retrieve_dependencies(config_path: impl AsRef<Path>) -> Result<(), Error> {
    let config_path = config_path.as_ref();
    if !config_path.exists() || config_path.is_dir() {
        return Ok(());
    }
    let config = config::load(config_path)?;
    let config_folder_path = config_path
        .parent()
        .expect("internal error: config path should have a parent");
    for (dep_name, dep_config) in config.dependencies {
        let target_parent_path = config_folder_path.join(TARGET_FOLDER_NAME);
        std::fs::create_dir_all(&target_parent_path)
            .map_err(|e| Error::Io(target_parent_path.clone(), e))?;
        let target_path = target_parent_path.join(&dep_name);
        if target_path.exists() {
            continue; // no-coverage (false negative)
        }
        let dep_path = dep_config
            .path
            .map(|path| config_folder_path.join(path))
            .filter(|path| path.is_dir() || dep_config.url.is_none());
        if let Some(dep_path) = dep_path {
            link_local_dependency(&target_path, &dep_path, &dep_name)?;
        } else if let Some(url) = dep_config.url {
            retrieve_url_dependency(&target_path, &url, &dep_name)?;
        } else {
            return Err(Error::NoDependencySource(dep_name));
        }
    }
    Ok(())
}

fn link_local_dependency(target_path: &Path, dep_path: &Path, dep_name: &str) -> Result<(), Error> {
    #[cfg(target_family = "unix")]
    {
        let source_path = std::env::current_dir()
            .map_err(|e| Error::Io("<current folder>".into(), e))?
            .join(dep_path.join(dep_name));
        std::os::unix::fs::symlink(&source_path, target_path).map_err(|e| Error::Io(source_path, e))
    }
    #[cfg(target_family = "windows")]
    {
        let source_path = std::env::current_dir()
            .map_err(|e| Error::Io("<current folder>".into(), e))?
            .join(dep_path.join(dep_name));
        std::os::windows::fs::symlink_dir(&source_path, target_path)
            .map_err(|e| Error::Io(source_path, e))
    }
    #[cfg(not(any(target_family = "unix", target_family = "windows")))]
    {
        copy_local_dependency(target_path, dep_path, dep_name)
    }
}

#[allow(dead_code)]
fn copy_local_dependency(target_path: &Path, dep_path: &Path, dep_name: &str) -> Result<(), Error> {
    let source_path = dep_path.join(dep_name);
    fs_extra::copy_items(
        &[&source_path],
        target_path,
        &CopyOptions::new().copy_inside(true),
    )
    .map(|_| ())
    .map_err(|e| Error::Copy(source_path, target_path.into(), e))
}

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
fn retrieve_url_dependency(target_path: &Path, url: &str, dep_name: &str) -> Result<(), Error> {
    let tmp_folder = tempfile::TempDir::new().map_err(|e| Error::Io("temp folder".into(), e))?;
    let zip_path = tmp_folder.path().join("files.zip");
    let extracted_path = tmp_folder.path().join("files");
    let mut response = reqwest::blocking::get(url).map_err(Error::Request)?;
    let mut zip_file =
        std::fs::File::create(&zip_path).map_err(|e| Error::Io(zip_path.clone(), e))?;
    std::io::copy(&mut response, &mut zip_file).map_err(|e| Error::Io(zip_path.clone(), e))?;
    zip::ZipArchive::new(
        std::fs::File::open(&zip_path).map_err(|e| Error::Io(zip_path.clone(), e))?,
    )
    .map_err(Error::Zip)?
    .extract(&extracted_path)
    .map_err(Error::Zip)?;
    let extracted_root_path = extracted_path
        .read_dir()
        .map_err(|e| Error::Io(zip_path.clone(), e))?
        .filter_map(Result::ok)
        .find(|entry| entry.path().is_dir())
        .map_or(extracted_path, |entry| entry.path());
    copy_local_dependency(target_path, &extracted_root_path, dep_name)
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn retrieve_url_dependency(_target_path: &Path, _url: &str, _dep_name: &str) -> Result<(), Error> {
    panic!("`wgso_deps` crate is only supported on Window, Linux and macOS")
}
