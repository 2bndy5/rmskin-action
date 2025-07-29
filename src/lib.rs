//! See description of [`CliArgs`] for basic info.
use std::{env, fs::OpenOptions, io::Write, path::PathBuf};
use tempfile::TempDir;

mod discover;
pub use discover::{HasComponents, discover_components};

mod cli;
pub use cli::{CliArgs, CliError};

pub(crate) mod file_utils;
pub use file_utils::{
    bitness::{Bitness, get_dll_bitness},
    header_img::validate_header_image,
    parse_ini::parse_rmskin_ini,
    zip::init_zip_for_package,
};

mod error;
pub use error::{ArchiveError, IniError, RmSkinBuildError};

mod logger;

#[cfg(any(feature = "py-binding", feature = "bin"))]
const DEBUG_ENV_TOGGLE: &str = "ACTIONS_STEP_DEBUG";

const GH_OUT_VAR_NAME: &str = "arc-name";

/// Helper function used in the `rmskin-build` binary executable.
pub fn main(cli_args: CliArgs) -> Result<(), RmSkinBuildError> {
    #[cfg(feature = "bin")]
    {
        use log::LevelFilter;

        logger::logger_init();
        let level = if env::var(DEBUG_ENV_TOGGLE).is_ok_and(|v| v == "true") {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        };
        log::set_max_level(level);
    }

    let project_path = cli_args
        .path
        .clone()
        .unwrap_or(PathBuf::from("./"))
        .canonicalize()?;
    {
        // canonicalize() will ensure file_name() is not None, thus unwrap() safely.
        let path_name = project_path.file_name().unwrap().to_string_lossy();
        log::info!("Searching path: {path_name}");
    }
    let components = discover_components(&project_path)?;
    if !components.is_valid() {
        return Err(RmSkinBuildError::MalformedProject);
    }

    let build_dir = TempDir::new()?; // uses an absolute path
    if components.rm_skin_bmp {
        validate_header_image(&project_path, build_dir.path())?;
    }
    let (arc_name, version) = parse_rmskin_ini(&cli_args, &project_path, build_dir.path())?;
    let archive_name = format!("{arc_name}_{version}.rmskin");
    init_zip_for_package(&archive_name, &cli_args, &project_path, build_dir.path())?;
    if let Ok(gh_out) = env::var("GITHUB_OUTPUT") {
        if let Ok(mut gh_out_file) = OpenOptions::new().append(true).open(gh_out) {
            writeln!(&mut gh_out_file, "{GH_OUT_VAR_NAME}={archive_name}")?;
        }
    } else {
        log::info!("Archive name: {archive_name}");
    }
    Ok(())
}

#[cfg(feature = "py-binding")]
use pyo3::prelude::*;

#[cfg(feature = "py-binding")]
#[cfg_attr(feature = "py-binding", pyfunction(name = "main"))]
fn main_py(py: Python) -> PyResult<()> {
    use clap::Parser;
    use pyo3::{exceptions::PyOSError, types::PyDict};

    let cli_args = CliArgs::parse_from(
        py.import("sys")?
            .getattr("argv")?
            .extract::<Vec<String>>()?,
    );

    let logging = py.import("logging")?;
    let format_str = "[%(levelname)5s]: %(message)s";
    let key_word_args = PyDict::new(py);
    key_word_args.set_item("format", format_str)?;
    logging.call_method("basicConfig", (), Some(&key_word_args))?;
    let level = logging.getattr(if env::var(DEBUG_ENV_TOGGLE).is_ok_and(|v| v == "true") {
        "DEBUG"
    } else {
        "INFO"
    })?;
    logging
        .call_method0("getLogger")?
        .call_method1("setLevel", (level,))?;

    main(cli_args).map_err(|e| PyOSError::new_err(e.to_string()))
}

#[cfg(feature = "py-binding")]
#[cfg_attr(feature = "py-binding", pymodule)]
fn rmskin_builder(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // enable log! passthrough to python logger
    pyo3_log::init();

    m.add_function(wrap_pyfunction!(discover::discover_components_py, m)?)?;
    m.add_function(wrap_pyfunction!(
        file_utils::parse_ini::parse_rmskin_ini_py,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        file_utils::header_img::validate_header_image_py,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(file_utils::bitness::is_dll_32, m)?)?;
    m.add_function(wrap_pyfunction!(
        file_utils::bitness::get_dll_bitness_py,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        file_utils::zip::init_zip_for_package_py,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(main_py, m)?)?;
    m.add_class::<CliArgs>()?;
    m.add_class::<Bitness>()?;
    m.add_class::<HasComponents>()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::{CliArgs, DEBUG_ENV_TOGGLE, GH_OUT_VAR_NAME, main};
    use ini::Ini;
    use std::{env, fs, path::PathBuf, str::FromStr};
    use tempfile::{NamedTempFile, TempDir};

    const FOOTER_LEN: usize = 16;

    fn run_main(with_gh_output: bool) {
        let dir_out = TempDir::new().unwrap();
        let gh_out_file = NamedTempFile::new_in(dir_out.path()).unwrap();
        let test_assets = PathBuf::from_str("tests/demo_project").unwrap();
        let mut cli_args = CliArgs::default();
        cli_args.dir_out = Some(dir_out.path().to_path_buf());
        cli_args.path = Some(test_assets);
        unsafe {
            env::set_var(DEBUG_ENV_TOGGLE, "true");
            if with_gh_output {
                env::set_var(
                    "GITHUB_OUTPUT",
                    gh_out_file.path().to_string_lossy().to_string(),
                );
            } else {
                env::remove_var("GITHUB_OUTPUT");
            }
        }
        assert!(main(cli_args).is_ok());
        let artifact = if with_gh_output {
            // check artifacts based on GITHUB_OUTPUT value
            let outputs = Ini::load_from_file(gh_out_file.path()).unwrap();
            let global_section: Option<String> = None;
            let arc_name = outputs
                .section(global_section)
                .unwrap()
                .get(GH_OUT_VAR_NAME)
                .unwrap();
            dir_out.path().to_path_buf().join(arc_name)
        } else {
            fs::read_dir(dir_out.path())
                .unwrap()
                .flatten()
                .find(|entry| {
                    let artifact = entry.path();
                    artifact
                        .extension()
                        .is_some_and(|v| v.to_string_lossy() == "rmskin")
                })
                .unwrap()
                .path()
        };
        let compressed_bytes = fs::read(&artifact).unwrap();
        assert!(compressed_bytes.len() > FOOTER_LEN);
        let (pkg, footer) = compressed_bytes.split_at(compressed_bytes.len() - FOOTER_LEN);
        assert!(footer.ends_with(b"\x00RMSKIN\x00"));
        let pkg_size = pkg.len() as u32;
        let embedded_size = u32::from_le_bytes([footer[0], footer[1], footer[2], footer[3]]);
        assert_eq!(pkg_size, embedded_size);
    }

    #[test]
    fn main_no_gh_output() {
        run_main(false);
    }

    #[test]
    fn main_with_gh_output() {
        run_main(true);
    }
}
