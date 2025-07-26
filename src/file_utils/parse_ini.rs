use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use super::RMSKIN_INI_NAME;
use crate::{CliArgs, IniError};
use ini::Ini;

#[cfg(feature = "py-binding")]
use pyo3::prelude::*;

#[cfg(feature = "py-binding")]
#[cfg_attr(feature = "py-binding", pyfunction)]
pub fn parse_rmskin_ini_py(
    cli_args: &CliArgs,
    path: PathBuf,
    build: PathBuf,
) -> PyResult<(String, String)> {
    use pyo3::exceptions::PyRuntimeError;

    parse_rmskin_ini(cli_args, &path, &build).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

const ROOT_SECTION_KEY: &str = "rmskin";
const VERSION_KEY: &str = "Version";
const AUTHOR_KEY: &str = "Author";
const NAME_KEY: &str = "Name";
const LOAD_TYPE_KEY: &str = "LoadType";
const DEFAULT_LOAD_TYPE: &str = "Skin";
const LOAD_KEY: &str = "Load";

/// Parses a RMSKIN.ini file. Returns a tuple of `(name, version)`.
///
/// The given `path` shall contain the the RMSKIN.ini file that is parsed.
/// This will also write an amended RMSKIN.ini file in the `build_dir` path.
pub fn parse_rmskin_ini(
    cli_args: &CliArgs,
    path: &Path,
    build_dir: &Path,
) -> Result<(String, String), IniError> {
    let mut rmskin_ini = Ini::load_from_file(path.to_path_buf().join(RMSKIN_INI_NAME))?;
    let root = rmskin_ini
        .section_mut(Some(ROOT_SECTION_KEY))
        .ok_or(IniError::MissingSection(ROOT_SECTION_KEY.to_string()))?;

    let author = if let Some(auth) = root.get(AUTHOR_KEY) {
        auth.to_string()
    } else {
        cli_args.get_author()?
    };
    root.insert(AUTHOR_KEY, &author);

    let version = if root
        .get(VERSION_KEY)
        .as_ref()
        .is_none_or(|ver| *ver == "auto")
    {
        cli_args.get_version()?
    } else {
        // unwrap() is ok because we checked for a None variant above
        root.get(VERSION_KEY).unwrap().to_owned()
    };
    root.insert(VERSION_KEY, &version);

    let arc_name = if let Some(name) = root.get(NAME_KEY) {
        name.to_string()
    } else {
        cli_args.get_title()?
    };
    root.insert(NAME_KEY, &arc_name);
    log::info!("Using Name '{arc_name}' and Version '{version}'");

    let load_t = root.get(LOAD_TYPE_KEY).unwrap_or(DEFAULT_LOAD_TYPE);
    if let Some(on_load) = root.get(LOAD_KEY) {
        let rel_path = PathBuf::from_str(&format!("{load_t}s"))
            .unwrap() // PathBuf::from_str() is actually infallible, so use unwrap() here.
            .join(on_load.escape_default().to_string().replace("\\", "/"));
        let on_load_path = path.to_path_buf().join(&rel_path);
        if let Err(err) = on_load_path.canonicalize() {
            log::error!("Failed to make absolute path to {on_load_path:?}");
            return Err(IniError::Io(err));
        }
    }

    rmskin_ini.write_to_file(build_dir.to_path_buf().join(RMSKIN_INI_NAME))?;

    Ok((arc_name, version))
}

#[cfg(test)]
mod test {
    use crate::file_utils::parse_ini::LOAD_KEY;

    use super::{
        AUTHOR_KEY, CliArgs, NAME_KEY, RMSKIN_INI_NAME, ROOT_SECTION_KEY, VERSION_KEY,
        parse_rmskin_ini,
    };
    use ini::Ini;
    use std::{path::PathBuf, str::FromStr};
    use tempfile::TempDir;

    #[test]
    fn parse_template() {
        let path = PathBuf::from_str("tests/demo_project").unwrap();
        let build = TempDir::new().unwrap();
        let cli_args = CliArgs::default();
        let (name, version) = parse_rmskin_ini(&cli_args, &path, build.path()).unwrap();

        assert_eq!(name, cli_args.get_title().unwrap());
        assert_eq!(version, cli_args.get_version().unwrap());
        let ini_path = build.path().to_path_buf().join(RMSKIN_INI_NAME);
        let ini_out = Ini::load_from_file(&ini_path).unwrap();
        let author = ini_out.get_from(Some(ROOT_SECTION_KEY), AUTHOR_KEY);
        assert_eq!(author, Some(cli_args.get_author().unwrap().as_str()));
    }

    const CUSTOM_AUTHOR: &str = "Unknown";
    const CUSTOM_VERSION: &str = "x0x.y0y";
    const CUSTOM_NAME: &str = "Test ini";

    fn parse_ini_custom(bad_on_load_val: bool) {
        let tmp_dir = TempDir::new().unwrap();
        let ini_in_path = tmp_dir.path().to_path_buf().join(RMSKIN_INI_NAME);
        let cli_args = CliArgs::default();
        let mut ini_in = Ini::new();
        let mut section = ini_in.with_section(Some(ROOT_SECTION_KEY));
        section.add(AUTHOR_KEY, CUSTOM_AUTHOR);
        section.add(VERSION_KEY, CUSTOM_VERSION);
        section.add(NAME_KEY, CUSTOM_NAME);
        if bad_on_load_val {
            section.add(LOAD_KEY, "MySkin/test.ini");
        }
        ini_in.write_to_file(&ini_in_path).unwrap();

        let result = parse_rmskin_ini(&cli_args, tmp_dir.path(), tmp_dir.path());
        match result {
            Ok((name, version)) => {
                assert_eq!(name, CUSTOM_NAME);
                assert_eq!(version, CUSTOM_VERSION);
                let ini_out = Ini::load_from_file(&ini_in_path).unwrap();
                let author = ini_out.get_from(Some(ROOT_SECTION_KEY), AUTHOR_KEY);
                assert_eq!(author, Some(CUSTOM_AUTHOR));
            }
            Err(_) => assert!(bad_on_load_val),
        }
    }

    #[test]
    fn parse_custom() {
        parse_ini_custom(false);
    }

    #[test]
    fn parse_bad_load_path() {
        parse_ini_custom(true);
    }
}
