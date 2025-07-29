use std::{env, path::PathBuf, process::Command, string::FromUtf8Error};
use thiserror::Error;

#[cfg(feature = "py-binding")]
use pyo3::prelude::*;

#[cfg(feature = "clap")]
use clap::Parser;

/// Errors emitted by various [`CliArgs`] functions.
#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Utf8Error(#[from] FromUtf8Error),

    #[error("Unknown working directory name")]
    UnknownWorkingDirectory,

    #[error("Malformed repository name: {0}")]
    MalformedRepoName(String),
}

/// A CLI tool to package Rainmeter Skins into a .rmskin file.
///
/// ## Ideal Repo Structure
///
/// The following files/folders are used if they exist in the project's root directory:
///
/// - `Skins/` (required): A folder to contain all necessary Rainmeter skins.
/// - `RMSKIN.ini` (required): A list of options specific to installing the skin(s).
/// - `RMSKIN.bmp` (optional): A header image to add brand recognition when installing
/// - `Layouts/` (optional): A folder that contains Rainmeter layout files.
/// - `Plugins/` (optional): A folder that contains Rainmeter plugins.
/// - `@Vault/` (optional): A resources folder accessible by all installed skins.
///   the generated rmskin file.
///
/// If none of the optional folders are present, then an error will be thrown.
/// Typically, the `Skins/` folder should be populated.
///
/// ### Repository template
///
/// A [cookiecutter repository](https://github.com/2bndy5/Rainmeter-Cookiecutter)
/// has also been created to quickly facilitate development of Rainmeter skins on Github.
#[cfg_attr(feature = "clap", derive(Parser, Debug, Default, Clone))]
#[cfg_attr(not(feature = "clap"), derive(Debug, Default, Clone))]
#[cfg_attr(
    feature = "clap",
    command(name = "rmskin-builder", about, long_about, verbatim_doc_comment)
)]
#[cfg_attr(feature = "py-binding", pyclass(module = "rmskin_builder"))]
pub struct CliArgs {
    /// The path to the git repository containing the Rainmeter project.
    #[cfg_attr(feature = "clap", arg(short, long, default_value = "./"))]
    pub path: Option<PathBuf>,

    /// The version number of the release.
    ///
    /// This will default to the git tag or the last 7 hexadecimal digits of the commit's SHA.
    #[cfg_attr(feature = "clap", arg(short = 'V', long))]
    version: Option<String>,

    /// The Author of the release.
    ///
    /// This will get its default value from (in order of precedence):
    /// - the environment variable `GITHUB_ACTOR`
    /// - the output from `git config get user.name`
    /// - "Unknown" when all else fails
    #[cfg_attr(feature = "clap", arg(short, long, verbatim_doc_comment))]
    author: Option<String>,

    /// Get the name of the Rainmeter project.
    ///
    /// This defaults to the repository name.
    /// If the environment variable `GITHUB_REPOSITORY` is not set, then
    /// this will just use the working directory's name.
    #[cfg_attr(feature = "clap", arg(short, long, verbatim_doc_comment))]
    title: Option<String>,

    /// The directory to package into an rmskin file.
    ///
    /// This defaults to the current working directory.
    #[cfg_attr(
        feature = "clap",
        arg(short, long, alias = "dir_out", default_value = "./")
    )]
    pub dir_out: Option<PathBuf>,
}

const GH_REPO: &str = "GITHUB_REPOSITORY";
const GH_REF: &str = "GITHUB_REF";
const GH_SHA: &str = "GITHUB_SHA";
const GH_ACTOR: &str = "GITHUB_ACTOR";

impl CliArgs {
    /// Get the version number of the release.
    ///
    /// This will default to the git tag or the last 7 hexadecimal digits of the commit's SHA.
    pub fn get_version(&self) -> Result<String, CliError> {
        if let Some(version) = &self.version {
            return Ok(version.clone());
        }
        if let Ok(gh_ref) = env::var(GH_REF) {
            if let Some(stripped) = gh_ref.strip_prefix("refs/tags/") {
                Ok(stripped.to_string())
            } else if let Ok(gh_sha) = env::var(GH_SHA) {
                let len = gh_sha.len().saturating_sub(7);
                Ok(gh_sha[len..].to_string())
            } else {
                Ok("x0x.y0y".to_string())
            }
        } else {
            // In case this is not running in a GitHub Action workflow:
            // Use `git` instead, but this assumes a non-shallow checkout.
            if let Ok(result) = Command::new("git").args(["describe", "--tags"]).output() {
                Ok(String::from_utf8(result.stdout.trim_ascii().to_vec())?)
            } else {
                let result = Command::new("git")
                    .args(["log", "-1", "--format=\"%h\""])
                    .output()?;
                Ok(String::from_utf8(result.stdout.trim_ascii().to_vec())?)
            }
        }
    }

    /// Get the name of the author for the release.
    ///
    /// This will get a default value from (in order of precedence):
    /// - the environment variable `GITHUB_ACTOR`
    /// - the output from `git config get user.name`
    /// - `"Unknown"` when all else fails
    pub fn get_author(&self) -> Result<String, CliError> {
        if let Some(author) = &self.author {
            Ok(author.clone())
        } else if let Ok(actor) = env::var(GH_ACTOR) {
            return Ok(actor);
        } else {
            let result = Command::new("git")
                .args(["config", "get", "user.name"])
                .output()?;
            return Ok(String::from_utf8(result.stdout.trim_ascii().to_vec())?);
        }
    }

    /// Get the Rainmeter project's title.
    ///
    /// This defaults to the repository name.
    /// If the environment variable `GITHUB_REPOSITORY` is not set, then
    /// this will just use the working directory's name.
    pub fn get_title(&self) -> Result<String, CliError> {
        if let Some(title) = &self.title {
            Ok(title.to_owned())
        } else {
            if let Ok(mut repo) = env::var(GH_REPO) {
                let divider = repo
                    .find('/')
                    .ok_or(CliError::MalformedRepoName(repo.to_owned()))?
                    + 1;
                return Ok(repo.split_off(divider));
            }
            let curr_dir = env::current_dir()?;
            Ok(curr_dir
                .file_name()
                .ok_or(CliError::UnknownWorkingDirectory)?
                .to_string_lossy()
                .to_string())
        }
    }
}

impl CliArgs {
    /// Set the Rainmeter project's version number.
    pub fn version(&mut self, value: Option<String>) {
        self.version = value;
    }

    /// Set the name of the author for the release.
    pub fn author(&mut self, value: Option<String>) {
        self.author = value;
    }

    /// Set the Rainmeter project's title.
    pub fn title(&mut self, value: Option<String>) {
        self.title = value;
    }
}

#[cfg(feature = "py-binding")]
#[cfg_attr(feature = "py-binding", pymethods)]
impl CliArgs {
    #[getter("version")]
    pub fn get_version_py(&self) -> PyResult<String> {
        use pyo3::exceptions::{PyIOError, PyValueError};

        self.get_version().map_err(|e| match e {
            CliError::Io(err) => PyIOError::new_err(err.to_string()),
            CliError::Utf8Error(err) => PyValueError::new_err(err.to_string()),
            CliError::UnknownWorkingDirectory => PyValueError::new_err("Unknown working directory"),
            CliError::MalformedRepoName(err) => {
                PyValueError::new_err(format!("Repository named malformed: {err}"))
            }
        })
    }

    #[getter("author")]
    pub fn get_author_py(&self) -> PyResult<String> {
        use pyo3::exceptions::{PyIOError, PyValueError};

        self.get_author().map_err(|e| match e {
            CliError::Io(err) => PyIOError::new_err(err.to_string()),
            CliError::Utf8Error(err) => PyValueError::new_err(err.to_string()),
            CliError::UnknownWorkingDirectory => PyValueError::new_err("Unknown working directory"),
            CliError::MalformedRepoName(err) => {
                PyValueError::new_err(format!("Repository named malformed: {err}"))
            }
        })
    }

    #[getter("title")]
    pub fn get_title_py(&self) -> PyResult<String> {
        use pyo3::exceptions::{PyIOError, PyValueError};

        self.get_title().map_err(|e| match e {
            CliError::Io(err) => PyIOError::new_err(err.to_string()),
            CliError::Utf8Error(err) => PyValueError::new_err(err.to_string()),
            CliError::UnknownWorkingDirectory => PyValueError::new_err("Unknown working directory"),
            CliError::MalformedRepoName(err) => {
                PyValueError::new_err(format!("Repository named malformed: {err}"))
            }
        })
    }

    #[setter]
    pub fn set_version(&mut self, value: Option<String>) {
        self.version(value);
    }

    #[setter]
    pub fn set_author(&mut self, value: Option<String>) {
        self.author(value);
    }

    #[setter]
    pub fn set_title(&mut self, value: Option<String>) {
        self.title(value);
    }

    pub fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    #[new]
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod test {
    use super::{CliArgs, CliError, GH_ACTOR, GH_REF, GH_REPO, GH_SHA};
    use std::env;

    #[test]
    fn version() {
        unsafe {
            env::remove_var(GH_REF);
            env::remove_var(GH_SHA);
        }
        let mut args = CliArgs::default();
        args.version(Some(args.get_version().unwrap()));
        assert!(!args.get_version().unwrap().is_empty());
    }

    #[test]
    fn version_ci_push() {
        let sha = "DEADBEEF";
        unsafe {
            env::set_var(GH_REF, "");
            env::set_var(GH_SHA, sha);
        }
        let args = CliArgs::default();
        assert!(sha.ends_with(&args.get_version().unwrap()));
    }

    #[test]
    fn version_ci_tag() {
        let tag = "v1.2.3";
        unsafe {
            env::set_var(GH_REF, format!("refs/tags/{tag}").as_str());
            env::remove_var(GH_SHA);
        }
        let args = CliArgs::default();
        assert_eq!(args.get_version().unwrap().as_str(), tag);
    }

    #[test]
    fn version_ci_default() {
        let tag = "x0x.y0y";
        unsafe {
            env::set_var(GH_REF, "");
            env::remove_var(GH_SHA);
        }
        let args = CliArgs::default();
        assert_eq!(args.get_version().unwrap().as_str(), tag);
    }

    #[test]
    fn author() {
        unsafe {
            env::remove_var(GH_ACTOR);
        }
        let mut args = CliArgs::default();
        args.author(Some(args.get_author().unwrap()));
        // `git config get user.name` can have various output.
        // Just test the value is not empty.
        assert!(!args.get_author().unwrap().is_empty());
    }

    #[test]
    fn author_ci() {
        let author = "2bndy5";
        unsafe {
            env::set_var(GH_ACTOR, author);
        }
        let args = CliArgs::default();
        assert_eq!(args.get_author().unwrap().as_str(), author);
    }

    #[test]
    fn title() {
        unsafe {
            env::remove_var(GH_REPO);
        }
        let mut args = CliArgs::default();
        args.title(Some(args.get_title().unwrap()));
        assert_eq!(
            args.get_title().unwrap().as_str(),
            env::current_dir()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );
    }

    #[test]
    fn title_ci() {
        unsafe {
            env::set_var(GH_REPO, "2bndy5/rmskin-action");
        }
        let args = CliArgs::default();
        assert_eq!(args.get_title().unwrap(), "rmskin-action".to_string());
    }

    #[test]
    fn title_ci_bad() {
        let bad_repo_name = "2bndy5\\rmskin-action";
        unsafe {
            env::set_var(GH_REPO, bad_repo_name);
        }
        let args = CliArgs::default();
        let title = args.get_title();
        assert!(title.is_err());
        if let Err(CliError::MalformedRepoName(bad_name)) = title {
            assert_eq!(&bad_name, bad_repo_name);
        }
    }
}
