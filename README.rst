
.. image:: https://github.com/2bndy5/rmskin-action/workflows/CI/badge.svg
    :target: https://github.com/2bndy5/rmskin-action/actions


rmskin-action
=============

A Python-based Github action tool to package a Repository's Rainmeter Content into a validating .rmskin file for Rainmeter's Skin Installer

.. Input Arguments
.. ===============

..         * ``version`` : (Optional) Version of the Rainmeter rmskin package.
..         * ``title`` : (Optional) Name of the Rainmeter rmskin package.
..         * ``author`` : (Optional) Account Username maintaining the rmskin package.
..         * ``path`` : (Optional) Base directory of repo being packaged.

Output Arguments
================

    * ``arc_name`` : The Path to & name of the generated rmskin file.

Example Usage
=============

.. code-block:: yaml
    
    jobs:  
      Build:
        runs-on: ubuntu-latest
        steps:
          # Checkout code
          - name: Checkout this Repo
            uses: actions/checkout@v2
          
          # # print contents of GITHUB_WORKSPACE dir
          # - name: verify contents
          #   run: ls $GITHUB_WORKSPACE

          # Runs a this repo's action
          - name: Run Build action
            id: builder
            uses: 2bndy5/rmskin-action@master

          # Use the output from the `builder` step
          - name: Print the output path & filename
            run: echo "The output file was ${{ steps.builder.outputs.arc_name }}"
