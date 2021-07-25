
.. image:: https://github.com/2bndy5/rmskin-action/workflows/CI/badge.svg
    :target: https://github.com/2bndy5/rmskin-action/actions
.. image:: https://img.shields.io/pypi/v/rmskin-builder.svg
    :target: https://pypi.python.org/pypi/rmskin-builder
    :alt: latest version on PyPI

rmskin-action
=============

A Python-based Github action tool to package a Repository's Rainmeter Content into a validating
.rmskin file for Rainmeter's Skin Installer.

.. important::
    If the repository contains a RMSKIN.bmp image to used as a header image in the rmskin package,
    then it must be using 24-bit colors. Additionally, if the image is not exactly 400x60, then
    this action's python script will resize it accordingly.

rmskin-builder Python package
-----------------------------

This action's *rmskin-builder.py* is now also available as a Python executable script via PyPI.
However, it is important that your Python installation's *Scripts* folder is found in your
Operating System's environment variable ``PATH``. If you're using a Python virtual envirnment,
then the *Scripts* folder does not need to be in your Operating System's environment variable
``PATH``.

.. code-block:: shell

    pip install rmskin-builder
    rmskin-builder.exe --help

Input Arguments
===============

.. csv-table::
    :header: "Argument", "Description", "Required"
    :widths: 5, 15, 3

    "version", "Version of the Rainmeter rmskin package. Defaults to last 8 digits of SHA from commit or ref/tags or otherwise 'x0x.x0xy'.", "no"
    "title", "Name of the Rainmeter rmskin package. Defaults to name of repository or otherwise the last directory in the ``path`` argument.", "no"
    "author", "Account Username maintaining the rmskin package. Defaults to Username that triggered the action or otherwise 'Unknown'.", "no"
    "path", "Base directory of repo being packaged. Defaults to current working path", "no"
    "dir_out", "Path to save generated rmskin package. Defaults to current working path", "no"
.. note::
    You can use your repository's ``RMSKIN.ini`` file to override any above inputs except ``dir_out`` & ``path`` inputs.

The above arguments are used as CLI arguments to the *rmskin_builder.py* script, but remember to
append the CLI arguments' name with a ``--``. For example, setting the ``path`` argument to use a
relative directory called *tests*:

.. code-block:: shell

    rmskin-builder.exe --path tests

Output Arguments
================

* ``arc_name`` : The name of the generated rmskin file saved in the
  path specified by ``dir_out`` input argument.

If executing the *rmskin_builder.py* script when not in a Github Action Runner, then this output
argument will show in the script's log output (& not saved anywhere).

Ideal Repo Structure
====================

- root directory

  - ``Skins``       a folder to contain all necessary Rainmeter skins
  - ``RMSKIN.ini``  list of options specific to installing the skin(s)
  - ``Layouts``     a folder that contains Rainmeter layout files
  - ``Plugins``     a folder that contains Rainmeter plugins
  - ``@Vault``      resources folder accessible by all installed skins

.. tip::
    `A cookiecutter repository <https://github.com/2bndy5/Rainmeter-Cookiecutter>`_
    has also been created to facilitate development of Rainmeter skins on Github
    quickly.

Example Usage
=============

.. code-block:: yaml

    name: RMSKIN Packager

    on:
      push:
      pull_request:
      release:
        types: [published]

    jobs:
      Build_n_Release:
        runs-on: ubuntu-latest

        steps:
          # Checkout code
          - name: Checkout this Repo
            uses: actions/checkout@v2

          # Runs a rmskin packager action
          - name: Run Build action
            id: builder
            uses: 2bndy5/rmskin-action@v1.1.6

          # Upload the asset (using the output from the `builder` step)
          - name: Upload Release Asset
            if: github.event_name == 'release'
            uses: csexton/release-asset-action@master
            with:
              file: "${{ steps.builder.outputs.arc_name }}"
              github-token: ${{ secrets.GITHUB_TOKEN }}
