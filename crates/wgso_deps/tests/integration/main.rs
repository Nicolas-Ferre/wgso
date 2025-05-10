#![allow(missing_docs, clippy::unwrap_used)]

use std::path::Path;
use wgso_deps::Error;

#[test]
fn retrieve_valid_dependencies() {
    let config_path = Path::new("tests/configs/valid");
    let result = wgso_deps::retrieve_dependencies(config_path.join("wgso.yaml"));
    let is_local_dep_retrieved = config_path.join("_/config/directives.yaml").is_file();
    let is_url_dep_retrieved = config_path.join("_/.github/dependabot.yml").is_file();
    fs_extra::remove_items(&[config_path.join("_")]).unwrap();
    assert!(result.is_ok());
    assert!(is_local_dep_retrieved);
    assert!(is_url_dep_retrieved);
}

#[test]
fn retrieve_valid_dependencies_with_url_fallback() {
    let config_path = Path::new("tests/configs/valid_fallback");
    let result = wgso_deps::retrieve_dependencies(config_path.join("wgso.yaml"));
    let is_dep_retrieved = config_path.join("_/.github/dependabot.yml").is_file();
    fs_extra::remove_items(&[config_path.join("_")]).unwrap();
    assert!(result.is_ok());
    assert!(is_dep_retrieved);
}

#[test]
fn retrieve_dependencies_with_no_source() {
    let config_path = Path::new("tests/configs/no_source");
    let result = wgso_deps::retrieve_dependencies(config_path.join("wgso.yaml"));
    let is_dep_retrieved = config_path.join("_/dep").exists();
    fs_extra::remove_items(&[config_path.join("_")]).unwrap();
    assert!(matches!(result, Err(Error::NoDependencySource(_))));
    assert!(!is_dep_retrieved);
}

#[test]
fn retrieve_dependencies_from_missing_config_file() {
    let config_path = Path::new("tests/configs/missing");
    let result = wgso_deps::retrieve_dependencies(config_path.join("wgso.yaml"));
    let is_folder_created = config_path.join("_").exists();
    fs_extra::remove_items(&[config_path.join("_")]).unwrap();
    assert!(result.is_ok());
    assert!(!is_folder_created);
}
