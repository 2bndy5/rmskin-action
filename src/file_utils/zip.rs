use std::{
    fs,
    io::{Cursor, Read, Seek, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::{
    ArchiveError, Bitness, CliArgs, HasComponents,
    file_utils::{RMSKIN_BMP_NAME, RMSKIN_INI_NAME},
    get_dll_bitness,
};

fn compress_file<W: Write + Seek>(
    archive: &mut ZipWriter<W>,
    dest: &Path,
    src: &Path,
) -> Result<(), ArchiveError> {
    log::debug!("Archiving file: {dest:?}");
    archive.start_file_from_path(
        dest,
        SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(9)),
    )?;
    let mut file_in = fs::File::open(src)?;
    let mut buf = vec![];
    file_in.read_to_end(&mut buf)?;
    let _ = archive.write(&buf)?;
    Ok(())
}

fn compress_folder<W: Write + Seek>(
    archive: &mut ZipWriter<W>,
    src: &Path,
    prefix: &Path,
) -> Result<(), ArchiveError> {
    archive.add_directory_from_path(
        src.strip_prefix(prefix)?,
        SimpleFileOptions::default()
            .compression_level(Some(9))
            .compression_method(CompressionMethod::Deflated),
    )?;
    for entry in fs::read_dir(src)?.flatten() {
        let entry = entry.path();
        if entry.is_dir() {
            compress_folder(archive, &entry, prefix)?;
        } else {
            let archive_dest = entry.strip_prefix(prefix)?;
            compress_file(archive, archive_dest, &entry)?;
        }
    }
    Ok(())
}

fn compress_plugins<W: Write + Seek>(
    archive: &mut ZipWriter<W>,
    src: &Path,
    prefix: &Path,
) -> Result<(), ArchiveError> {
    let compression_opts = SimpleFileOptions::default()
        .compression_level(Some(9))
        .compression_method(CompressionMethod::Deflated);
    archive.add_directory_from_path(src.to_path_buf().strip_prefix(prefix)?, compression_opts)?;
    archive.add_directory_from_path(
        src.to_path_buf().join("32bit").strip_prefix(prefix)?,
        compression_opts,
    )?;
    archive.add_directory_from_path(
        src.to_path_buf().join("64bit").strip_prefix(prefix)?,
        compression_opts,
    )?;
    for entry in fs::read_dir(src)?.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            for plugin in fs::read_dir(&entry_path)?.flatten() {
                let plugin_path = plugin.path();
                // Each plugin gets its own directory within Plugins/ folder.
                if plugin_path.is_dir() {
                    // Plugins (actual DLL files) should be in this path.
                    for dll in fs::read_dir(&plugin_path)?.flatten() {
                        let dll_path = dll.path();
                        if dll_path.is_file() {
                            let bitness = get_dll_bitness(&dll_path)?;
                            let dest_sub_path = match bitness {
                                Bitness::Bit32() => "32bit",
                                Bitness::Bit64() => "64bit",
                                Bitness::Unknown(machine_type) => {
                                    return Err(ArchiveError::InvalidPlugin(machine_type));
                                }
                            };
                            let dest = src
                                .to_path_buf()
                                .join(dest_sub_path)
                                // `PathBuf::file_name()` is Infallible, so use unwrap() here.
                                .join(dll_path.file_name().unwrap());
                            compress_file(archive, dest.strip_prefix(prefix)?, &dll_path)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Generate the .rmskin file in the given `build_dir`.
///
/// This uses the `archive_name` for the rmskin file.
/// All applicable contents found in the given `path` are
/// included in the rmskin file.
///
/// Note, the rmskin file is basically a zip file with
/// custom data appended to the compressed binary data.
pub fn init_zip_for_package(
    archive_name: &str,
    args: &CliArgs,
    path: &Path,
    build_dir: &Path,
) -> Result<PathBuf, ArchiveError> {
    let mut buf = vec![];
    let mut archive = ZipWriter::new(Cursor::new(&mut buf));

    // PathBuf::from_str() is actually infallible, so use unwrap() here.
    let ini_dest = PathBuf::from_str(RMSKIN_INI_NAME).unwrap();
    let ini_src = build_dir.to_path_buf().join(RMSKIN_INI_NAME);
    compress_file(&mut archive, &ini_dest, &ini_src)?;

    let bmp_src = build_dir.to_path_buf().join(RMSKIN_BMP_NAME);
    if bmp_src.exists() {
        // PathBuf::from_str() is actually infallible, so use unwrap() here.
        let bpm_dest = PathBuf::from_str(RMSKIN_BMP_NAME).unwrap();
        compress_file(&mut archive, &bpm_dest, &bmp_src)?;
    }

    // we only care about any applicable directories present
    for entry in fs::read_dir(path)?.flatten() {
        let entry = entry.path();
        if entry.is_dir() {
            // add folder recursively to archive
            if let Some(folder_name) = entry.file_name() {
                let folder_name = folder_name.to_string_lossy().to_string();
                match folder_name.as_str() {
                    HasComponents::SKINS | HasComponents::LAYOUTS | HasComponents::VAULT => {
                        compress_folder(&mut archive, &entry, path)?
                    }
                    HasComponents::PLUGINS => compress_plugins(&mut archive, &entry, path)?,
                    _ => (),
                }
            }
        }
    }
    archive.finish()?;

    let buf_len = buf.len() as u64;
    log::info!("Archive size = {buf_len} ({buf_len:#X})");
    let mut custom_footer = [0u8; 16];
    custom_footer[0..8].copy_from_slice(&buf_len.to_le_bytes());
    custom_footer[8..16].copy_from_slice(b"\x00RMSKIN\x00");
    log::debug!("Appending footer: {custom_footer:X?}");
    buf.extend_from_slice(&custom_footer);

    let out_path = args
        .dir_out
        .clone()
        // PathBuf::from_str() is actually infallible, so use unwrap() here.
        .unwrap_or(PathBuf::from_str("./").unwrap())
        .join(archive_name);
    let mut file = fs::File::create(&out_path)?;
    file.write_all(&buf)?;
    file.flush()?;
    log::info!("Archive successfully prepared.");
    Ok(out_path.canonicalize()?)
}

#[cfg(feature = "py-binding")]
use pyo3::prelude::*;

#[cfg(feature = "py-binding")]
#[cfg_attr(feature = "py-binding", pyfunction(name = "init_zip_for_package"))]
pub fn init_zip_for_package_py(
    arc_name: String,
    args: CliArgs,
    path: PathBuf,
    build_dir: PathBuf,
) -> PyResult<PathBuf> {
    use pyo3::exceptions::PyOSError;

    init_zip_for_package(&arc_name, &args, &path, &build_dir)
        .map_err(|e| PyOSError::new_err(e.to_string()))
}
