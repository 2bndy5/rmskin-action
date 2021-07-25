
.. image:: https://github.com/2bndy5/rmskin-action/workflows/CI/badge.svg
    :target: https://github.com/2bndy5/rmskin-action/actions

rmskin-action
=============

A Python-based Github action tool to package a Repository's Rainmeter Content into a validating .rmskin file for Rainmeter's Skin Installer.

.. important::
    If the repository contains a RMSKIN.bmp image to used as a header image in the rmskin package, then it must be using 24-bit colors.
    Additionally, if the image is not exactly 400x60, then this action's python script will resize it accordingly.

Input Arguments
===============

.. csv-table::
    :header: "Argument", "Description", "Required"
    :widths: 5, 15, 3

    "version", "Version of the Rainmeter rmskin package. Defaults to last 8 digits of SHA from commit or ref/tags", "no"
    "title", "Name of the Rainmeter rmskin package. Defaults to name of repository", "no"
    "author", "Account Username maintaining the rmskin package. Defaults to Username that owns the repository.", "no"
    "path", "Base directory of repo being packaged. Defaults to workflow's workspace path", "no"
    "dir_out", "Path to save generated rmskin package. Defaults to workflow's workspace path", "no"
.. note::
    You can use your repository's ``RMSKIN.ini`` file to override any above inputs except ``dir_out`` & ``path`` inputs.

Output Arguments
================

* ``arc_name`` : The name of the generated rmskin file saved in the
  path specified by ``dir_out`` input argument.

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
            types:
                - published

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
              uses: 2bndy5/rmskin-action@v1.1.2

            # Use the output from the `builder` step
            - name: Print the output filename
              run: echo "The output file was ${{ steps.builder.outputs.arc_name }}"

            # get release upload_url
            - name: Get Release
              id: get_release
              uses: bruceadams/get-release@v1.2.0
              if: github.event_name == 'release'
              env:
                GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

            # Upload the asset
            - name: Upload Release Asset
              id: upload-release-asset
              uses: actions/upload-release-asset@v1
              if: github.event_name == 'release'
              env:
                GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
              with:
                upload_url: ${{ steps.get_release.outputs.upload_url }}
                asset_path: ./${{ steps.builder.outputs.arc_name }}
                asset_name: ${{ steps.builder.outputs.arc_name }}
                asset_content_type: application/zip
