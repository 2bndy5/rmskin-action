use std::path::StripPrefixError;

use image::ImageError;
use thiserror::Error;

use crate::CliError;

/// Errors emitted by [`parse_rmskin_ini()`](fn@crate::parse_rmskin_ini).
#[derive(Debug, Error)]
pub enum IniError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse ini syntax: {0}")]
    ParseError(#[from] ini::Error),

    #[error("Missing {0} section in INI file")]
    MissingSection(String),

    #[error("Missing specified file loaded upon install: {0}")]
    MissingOnLoad(String),

    #[error("{0}")]
    CliError(#[from] CliError),
}

/// Errors emitted by [`init_zip_for_package()`](fn@crate::init_zip_for_package).
#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    StripPrefixError(#[from] StripPrefixError),

    #[error("{0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("invalid machine type ({0}) found for alleged plugin")]
    InvalidPlugin(u16),
}

/// Errors emitted by [`main()`][fn@crate::main].
#[derive(Debug, Error)]
pub enum RmSkinBuildError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    ImageError(#[from] ImageError),

    #[error("{0}")]
    IniError(#[from] IniError),

    #[error("{0}")]
    CliError(#[from] CliError),

    #[error("{0}")]
    ArchiveError(#[from] ArchiveError),

    #[error("Project is malformed. It must contain a RMSKIN.ini and a populated Skins folder")]
    MalformedProject,
}
