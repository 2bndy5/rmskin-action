rmskin-action
=============

A Python-based Github action tool to package a Repository's Rainmeter Content into a validating .rmskin file for Rainmeter's Skin Installer

Input Arguments
===============

        * ``version`` : (Optional) Version of the Rainmeter rmskin package.
        * ``title`` : (Optional) Name of the Rainmeter rmskin package.
        * ``author`` : (Optional) Account Username maintaining the rmskin package.
        * ``path`` : (Optional) Base directory of repo being packaged.

Output Arguments
================

    * ``arc_name`` : The Path to & name of the generated rmskin file.

Example Usage
=============

.. code-block:: yaml
    
    jobs:
        # This workflow contains a single job called "build"
        build:
            # The type of runner that the job will run on
            runs-on: ubuntu-latest

            # Steps represent a sequence of tasks that will be executed as part of the job
            steps:
            - name: checkout a test repo of a rainmeter skin
              # Checks-out a repository under $GITHUB_WORKSPACE, so your job can access it
              uses: actions/checkout@v2
              with:
                  repositoy: 2bndy5/Goo-e-Rainmeter-Skin 

            # Runs a this repo's action
            - name: Run Build action
              id: builder
              uses: 2bndy5/rmskin-action

            # Use the output from the `builder` step
            - name: Print the output path & filename
              run: echo "The output file was ${{ steps.builder.outputs.arc_name }}"
