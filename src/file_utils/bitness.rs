use std::{
    fs::File,
    io::{self, Read, Seek},
    path::Path,
};

#[cfg(feature = "py-binding")]
use pyo3::prelude::*;
#[cfg(feature = "py-binding")]
use std::path::PathBuf;

/// An enumeration of possible output values from [`get_dll_bitness()`](fn@crate::get_dll_bitness).
#[cfg_attr(feature = "py-binding", pyclass(module = "rmskin_builder", eq))]
#[derive(Debug, PartialEq, Eq)]
pub enum Bitness {
    Bit32(),
    Bit64(),
    Unknown(u16),
}

#[cfg(feature = "py-binding")]
#[cfg_attr(feature = "py-binding", pyfunction)]
pub fn is_dll_32(py: Python, dll_file: PathBuf) -> PyResult<bool> {
    use pyo3::exceptions::{PyDeprecationWarning, PyIOError, PyRuntimeError};

    PyDeprecationWarning::new_err(
        "rmskin_builder.is_dll_32() is deprecated. \
        Use rmskin_builder.get_dll_bitness() for better error handling",
    )
    .print(py);

    let bitness = get_dll_bitness(&dll_file).map_err(|e| PyIOError::new_err(e.to_string()))?;
    match bitness {
        Bitness::Bit32() => Ok(true),
        Bitness::Bit64() => Ok(false),
        Bitness::Unknown(machine_type) => Err(PyRuntimeError::new_err(format!(
            "Unknown machine type ({machine_type}) for DLL ({dll_file:?})"
        ))),
    }
}

#[cfg(feature = "py-binding")]
#[cfg_attr(feature = "py-binding", pyfunction(name = "get_dll_bitness"))]
pub fn get_dll_bitness_py(path: PathBuf) -> PyResult<Bitness> {
    use pyo3::exceptions::PyIOError;

    get_dll_bitness(&path).map_err(|e| PyIOError::new_err(e.to_string()))
}

/// Get the "bitness" of the DLL pointed to by `path`.
pub fn get_dll_bitness(path: &Path) -> io::Result<Bitness> {
    let mut file = File::open(path)?;
    let mut header_bytes = [0; 64]; // Enough to read the DOS header and PE signature offset
    file.read_exact(&mut header_bytes)?;

    // Get the offset to the PE signature from the DOS header (e_lfanew)
    let pe_offset = u32::from_le_bytes([
        header_bytes[0x3c],
        header_bytes[0x3d],
        header_bytes[0x3e],
        header_bytes[0x3f],
    ]) as usize;

    // Seek to the PE signature
    file.seek(io::SeekFrom::Start(pe_offset as u64))?;
    let mut pe_signature = [0; 4];
    file.read_exact(&mut pe_signature)?;

    // Check if it's a valid PE file ("PE\0\0")
    if &pe_signature != b"PE\0\0" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Not a valid PE file",
        ));
    }

    // Read the IMAGE_FILE_HEADER (Machine field is at offset 4 relative to the PE signature)
    let mut file_header_bytes = [0; 2]; // Size of IMAGE_FILE_HEADER
    file.read_exact(&mut file_header_bytes)?;

    let machine = u16::from_le_bytes(file_header_bytes);

    match machine {
        0x014c => Ok(Bitness::Bit32()), // IMAGE_FILE_MACHINE_I386
        0x8664 => Ok(Bitness::Bit64()), // IMAGE_FILE_MACHINE_AMD64
        _ => Ok(Bitness::Unknown(machine)),
    }
}

#[cfg(test)]
mod test {
    use super::{Bitness, get_dll_bitness};
    use std::{fs, path::PathBuf, str::FromStr};
    use tempfile::NamedTempFile;

    fn run_test(asset: &str) -> std::io::Result<Bitness> {
        let asset = PathBuf::from_str(asset).unwrap();
        get_dll_bitness(&asset)
    }

    #[test]
    fn validate_32bit() {
        assert!(matches!(
            run_test("tests/demo_project/Plugins/Test/32bit/ConfigActive.dll"),
            Ok(Bitness::Bit32())
        ));
    }

    #[test]
    fn validate_64bit() {
        assert!(matches!(
            run_test("tests/demo_project/Plugins/Test/64bit/ConfigActive.dll"),
            Ok(Bitness::Bit64())
        ));
    }

    fn invalid_binary(bad_header_val: bool) -> std::io::Result<Bitness> {
        let tmp_bin_path = NamedTempFile::new().unwrap();
        let mut buf = [0; 80];
        let offset = 0x40_u32;
        buf[0x3c..0x40].copy_from_slice(&offset.to_le_bytes());
        if bad_header_val {
            buf[0x40..0x44].copy_from_slice(&[0xFF; 4]);
        } else {
            buf[0x40..0x44].copy_from_slice(b"PE\x00\x00");
            // an invalid machine type
            buf[0x44..0x46].copy_from_slice(&0_u16.to_le_bytes());
        }
        fs::write(&tmp_bin_path, buf).unwrap();
        get_dll_bitness(tmp_bin_path.path())
    }

    #[test]
    fn bad_bin_header() {
        let result = invalid_binary(true);
        assert!(result.is_err());
    }

    #[test]
    fn bad_machine_type() {
        let result = invalid_binary(false);
        assert!(matches!(result, Ok(Bitness::Unknown(0))));
    }
}
