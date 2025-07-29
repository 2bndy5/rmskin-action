use std::env;

use anyhow::{Result, anyhow};
use clap::Parser;
use rmskin_builder::{CliArgs, main as run_main};

#[cfg(test)]
const TEST_BUILD_DIR_ENV_KEY: &str = "RMSKIN_BUILDER_TEST_BUILD_DIR";

fn main() -> Result<()> {
    let cli_args = CliArgs::parse_from(
        #[cfg(test)]
        vec![
            "rmskin-build",
            "--path",
            "tests/demo_project",
            "--dir-out",
            env::var(TEST_BUILD_DIR_ENV_KEY)
                .unwrap()
                .replace("\\", "/")
                .as_str(),
        ],
        #[cfg(not(test))]
        env::args(),
    );
    run_main(cli_args).map_err(|e| anyhow!("Failed to assemble rmskin file: {e:?}"))
}

#[cfg(test)]
mod test {
    use super::{TEST_BUILD_DIR_ENV_KEY, main};
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn run_main() {
        let tmp_dir = TempDir::new().unwrap();
        unsafe {
            env::set_var(
                TEST_BUILD_DIR_ENV_KEY,
                tmp_dir.path().to_string_lossy().to_string(),
            );
        }
        assert!(main().is_ok());
    }
}
