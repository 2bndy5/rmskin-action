use image::{ColorType, ExtendedColorType, ImageError, imageops::FilterType};
use std::path::Path;
#[cfg(feature = "py-binding")]
use std::path::PathBuf;

#[cfg(feature = "py-binding")]
use pyo3::prelude::*;

use super::RMSKIN_BMP_NAME;

#[cfg(feature = "py-binding")]
#[cfg_attr(feature = "py-binding", pyfunction(name = "validate_header_image"))]
pub fn validate_header_image_py(path: PathBuf, build_dir: PathBuf) -> PyResult<()> {
    use pyo3::exceptions::PyRuntimeError;

    validate_header_image(&path, &build_dir).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Validate the RMSKIN.bmp file in the given `path`.
///
/// This also copies the validated image to the given `build_dir`.
pub fn validate_header_image(path: &Path, build_dir: &Path) -> Result<(), ImageError> {
    let img_in = path.to_path_buf().join(RMSKIN_BMP_NAME);
    log::debug!("Checking {img_in:?}");
    let mut img = image::open(&img_in)?;
    if img.width() != 400 || img.height() != 60 {
        log::warn!("Resizing header image to 400x60");
        img = img.resize_exact(400, 60, FilterType::Nearest);
    }
    if img.color() != ColorType::Rgb8 {
        log::warn!("Correcting the color space in the header image");
    }
    let out_path = build_dir.to_path_buf().join(RMSKIN_BMP_NAME);
    image::save_buffer(
        &out_path,
        &img.into_rgb8(),
        400,
        60,
        ExtendedColorType::Rgb8,
    )?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::{ImageError, validate_header_image};
    use crate::{file_utils::RMSKIN_BMP_NAME, logger::logger_init};
    use image::{ColorType, GrayImage};
    use std::{path::PathBuf, str::FromStr};
    use tempfile::TempDir;

    #[test]
    fn valid_bmp() {
        let asset = PathBuf::from_str("tests/demo_project").unwrap();
        let build_dir = TempDir::new().unwrap();
        validate_header_image(&asset, build_dir.path()).unwrap();
    }

    #[derive(Debug, Default)]
    struct InvalidTestParams {
        bad_in_path: bool,
        bad_out_path: bool,
    }

    fn test_invalid_img(test_params: InvalidTestParams) -> Result<TempDir, ImageError> {
        logger_init();
        log::set_max_level(log::LevelFilter::Debug);
        let build_dir = TempDir::new().unwrap();
        let img = GrayImage::new(350, 50);
        let asset = build_dir
            .path()
            .to_path_buf()
            .join(if test_params.bad_in_path {
                "test.bmp"
            } else {
                RMSKIN_BMP_NAME
            });
        img.save(&asset).unwrap();
        (if test_params.bad_out_path {
            validate_header_image(
                build_dir.path(),
                &build_dir.path().to_path_buf().join("build"),
            )
        } else {
            validate_header_image(build_dir.path(), build_dir.path())
        })
        .map(|_| build_dir)
    }

    #[test]
    fn invalid_bmp() {
        let tmp_dir = test_invalid_img(InvalidTestParams::default()).unwrap();
        let asset = tmp_dir.path().to_path_buf().join(RMSKIN_BMP_NAME);
        let bmp_img = image::open(&asset).unwrap();
        assert_eq!(bmp_img.height(), 60);
        assert_eq!(bmp_img.width(), 400);
        assert_eq!(bmp_img.color(), ColorType::Rgb8);
    }

    #[test]
    fn invalid_bmp_out_path() {
        let result = test_invalid_img(InvalidTestParams {
            bad_out_path: true,
            ..Default::default()
        });
        assert!(result.is_err());
    }

    #[test]
    fn invalid_bmp_in_path() {
        let result = test_invalid_img(InvalidTestParams {
            bad_in_path: true,
            ..Default::default()
        });
        assert!(result.is_err());
    }
}
