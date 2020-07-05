
.. image:: https://github.com/2bndy5/rmskin-action/workflows/CI/badge.svg
    :target: https://github.com/2bndy5/rmskin-action/actions


rmskin-action
=============

A Python-based Github action tool to package a Repository's Rainmeter Content into a validating .rmskin file for Rainmeter's Skin Installer

Input Arguments
===============

    Please use your repository's ``RMSKIN.ini`` file to override any default inputs

Output Arguments
================

    * ``arc_name`` : The Path to & name of the generated rmskin file.

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
          # - edited

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
            uses: 2bndy5/rmskin-action@master

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
              upload_url: ${{ steps.create_release.outputs.upload_url || steps.get_release.outputs.upload_url }}
              asset_path: ./${{ steps.builder.outputs.arc_name }}
              asset_name: ${{ steps.builder.outputs.arc_name }}
              asset_content_type: application/zip
     