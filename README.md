<!-- markdownlint-disable MD041 -->

[![action-ci-badge]][action-ci-link]
[![pypi-ci-badge]][pypi-ci-link]
[![rust-ci-badge]][rust-ci-link]
[![pypi-badge]][pypi-link]
[![pypi-stats-badge]][pypi-stats-link]
[![codecov-badge]][codecov-link]

[pypi-ci-badge]: https://github.com/2bndy5/rmskin-action/actions/workflows/python.yml/badge.svg
[pypi-ci-link]: https://github.com/2bndy5/rmskin-action/actions/workflows/python.yml
[action-ci-badge]: https://github.com/2bndy5/rmskin-action/actions/workflows/self-test.yml/badge.svg
[action-ci-link]: https://github.com/2bndy5/rmskin-action/actions/workflows/self-test.yml
[rust-ci-badge]: https://github.com/2bndy5/rmskin-action/actions/workflows/rust.yml/badge.svg
[rust-ci-link]: https://github.com/2bndy5/rmskin-action/actions/workflows/rust.yml
[pypi-badge]: https://img.shields.io/pypi/v/rmskin-builder.svg
[pypi-link]: https://pypi.python.org/pypi/rmskin-builder
[pypi-stats-badge]: https://static.pepy.tech/personalized-badge/rmskin-builder?period=total&units=international_system&left_color=grey&right_color=blue&left_text=PyPi%20Downloads
[pypi-stats-link]: https://pepy.tech/project/rmskin-builder
[codecov-badge]: https://codecov.io/github/2bndy5/rmskin-action/graph/badge.svg?token=825YGO53XJ
[codecov-link]: https://codecov.io/github/2bndy5/rmskin-action

# rmskin-action

A Github Action that packages a repository's Rainmeter content into a validating
.rmskin file for Rainmeter's Skin Installer.

## Deployments

There various ways to employ this software (written in Rust).

### Github Actions

```yaml
name: RMSKIN Packager

on:
  push:
    branches: [main]
    tags: '*'
  pull_request:
    branches: [main]

jobs:
  build-n-release:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout this Repo
        uses: actions/checkout@v4

      # Run this rmskin-action
      - name: Run Build action
        id: builder
        uses: 2bndy5/rmskin-action@v2.0.3

      # Upload the asset (using the output from the `builder` step)
      - name: Upload Release Asset
        if: startsWith(github.ref, 'refs/tags/')
        env:
          GITHUB_TOKEN: ${{ github.token }}
        run: gh release upload ${{ github.ref_name }} ${{ steps.builder.outputs.arc_name }}
```

### Python Package

Originally, this was written as a pure Python executable script.
After migrating the code base to Rust,
the Python package is still maintained as an FFI binding.

```shell
pip install rmskin-builder
rmskin-builder.exe --help
```

### Rust package

[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall

A Rust crate is published to take advantage of [cargo-binstall] for easily installing a portable binary executable.

```shell
cargo binstall rmskin-builder
rmskin-build --help
```

## Input/CLI options

| Option | Description | Required |
|--------|:------------|:---------|
| `path` | Base directory of repo being packaged. Defaults to current working path. | no |
| `dir-out` | Path to save generated rmskin package. Defaults to current working path. This can also be specified using `dir_out` for backward compatibility. | no |
| `version` | Version of the Rainmeter rmskin package. Defaults to last 7 digits of SHA from commit or ref/tags or otherwise `x0x.y0y`. | no |
| `title` | Name of the Rainmeter rmskin package. Defaults to name of repository or otherwise the last directory name in the `path` option. | no |
| `author` | Account Username maintaining the rmskin package. Defaults to Username that triggered the action or the `git config user.name`; `Unknown` when all else fails. | no |

> [!NOTE]
> You can use your project's `RMSKIN.ini` file to override any above inputs except `dir-out` and `path`.

The above arguments are also used as CLI arguments
but remember to prepend `--` to option's name (eg `path` becomes `--path`).

## Output Variables

- `arc-name`: The name of the generated rmskin file saved in the
  path specified by `dir_out` input argument.
- `arc_name`: The same as `arc-name` output value.
  This output variable only exists for backward compatibility.

When not executed in a Github Actions workflow, then this output variable will printed to
stdout as `Archive name: **.rmskin`.

## Ideal Package Structure

Ideally, the package directory (located at `path` input value) can have the following files/folders:

| Name | Description | Required |
|------|:------------|----------|
| `Skins`       | A folder to contain all necessary Rainmeter skins. | yes |
| `RMSKIN.ini`  | list of options specific to installing the skin(s). | yes |
| `RMSKIN.bmp`  | A brand/logo image displayed in the Rainmeter installer. | no |
| `Layouts`     | A folder that contains Rainmeter layout files. | no |
| `Plugins`     | A folder that contains Rainmeter plugins. | no |
| `@Vault`      | A resources folder accessible by all installed skins. | no |

> [!TIP]
> A [cookiecutter repository](https://github.com/2bndy5/Rainmeter-Cookiecutter)
> has also been created to facilitate development of Rainmeter skins on Github quickly.
