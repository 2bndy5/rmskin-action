use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(feature = "py-binding")]
use pyo3::prelude::*;

use crate::file_utils::{RMSKIN_BMP_NAME, RMSKIN_INI_NAME};

/// A data structure to represent the presence of a
/// Rainmeter project's distributable components.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    feature = "py-binding",
    pyclass(get_all, set_all, module = "rmskin_builder", eq)
)]
pub struct HasComponents {
    pub rm_skin_ini: bool,
    pub skins: i32,
    pub layouts: i32,
    pub plugins: bool,
    pub vault: i32,
    pub rm_skin_bmp: bool,
}

impl HasComponents {
    pub const VAULT: &str = "@Vault";
    pub const PLUGINS: &str = "Plugins";
    pub const LAYOUTS: &str = "Layouts";
    pub const SKINS: &str = "Skins";

    pub fn is_valid(&self) -> bool {
        self.rm_skin_ini && self.skins > 0
    }
}

#[cfg(feature = "py-binding")]
#[cfg_attr(feature = "py-binding", pymethods)]
impl HasComponents {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    #[pyo3(name = "is_valid")]
    pub fn is_valid_py(&self) -> bool {
        self.is_valid()
    }
}

fn count_dir_children(path: PathBuf, dirs: bool, files: bool) -> Result<i32, std::io::Error> {
    let mut count = 0;
    for entry in (fs::read_dir(path)?).flatten() {
        let entry_path = entry.path();
        if (entry_path.is_dir() && dirs) || (entry_path.is_file() && files) {
            count += 1;
        }
    }
    Ok(count)
}

/// The method that does preliminary discovery of rmskin package components.
#[cfg_attr(feature = "py-binding", pyfunction(name = "discover_components"))]
#[cfg(feature = "py-binding")]
pub fn discover_components_py(path: PathBuf) -> PyResult<HasComponents> {
    use pyo3::exceptions::PyIOError;

    discover_components(&path).map_err(|e| PyIOError::new_err(e.to_string()))
}

/// The method that does preliminary discovery of rmskin package components.
pub fn discover_components<P: AsRef<Path>>(path: P) -> Result<HasComponents, std::io::Error> {
    let mut components = HasComponents::default();
    for entry in fs::read_dir(path)? {
        let entry_path = entry?.path();
        if let Some(dir_name) = entry_path.file_name() {
            let name = dir_name.to_string_lossy().to_string();
            match name.as_str() {
                HasComponents::SKINS => {
                    let count = count_dir_children(entry_path, true, false)?;
                    log::info!("Found {count} possible skin(s)");
                    components.skins = count;
                }
                HasComponents::VAULT => {
                    let count = count_dir_children(entry_path, true, true)?;
                    log::info!("Found {count} possible @Vault item(s)");
                    components.vault = count;
                }
                HasComponents::PLUGINS => {
                    let count = count_dir_children(entry_path, true, false)?;
                    log::info!("Found {count} Plugins folder");
                    components.plugins = true;
                }
                HasComponents::LAYOUTS => {
                    let count = count_dir_children(entry_path, true, true)?;
                    log::info!("Found {count} possible layout(s)");
                    components.layouts = count;
                }
                RMSKIN_INI_NAME => {
                    log::info!("Found RMSKIN.ini file");
                    components.rm_skin_ini = true;
                }
                RMSKIN_BMP_NAME => {
                    log::info!("Found header image file");
                    components.rm_skin_bmp = true;
                }
                _ => log::debug!("Skipping directory, {name}"),
            }
        }
    }
    Ok(components)
}

#[cfg(test)]
mod test {
    use super::{HasComponents, discover_components};

    #[test]
    fn basic() {
        let components = discover_components("tests/demo_project").unwrap();
        assert!(components.is_valid());
        let expected = HasComponents {
            rm_skin_ini: true,
            skins: 1,
            layouts: 1,
            plugins: true,
            vault: 1,
            rm_skin_bmp: true,
        };
        assert_eq!(components, expected);
    }
}
