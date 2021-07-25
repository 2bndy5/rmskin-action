"""A setuptools based setup module.

See:
https://packaging.python.org/en/latest/distributing.html
https://github.com/pypa/sampleproject
"""
import os
from codecs import open as open_codec  # To use a consistent encoding
from setuptools import setup


ROOT_DIR = os.path.abspath(os.path.dirname(__file__))
REPO = "https://github.com/2bndy5/rmskin-action"

# Get the long description from the README file
with open_codec(os.path.join(ROOT_DIR, "README.rst"), encoding="utf-8") as f:
    long_description = f.read()

setup(
    name="rmskin-action",
    use_scm_version=True,
    setup_requires=["setuptools_scm"],
    description="A script that will attempt to assemble a validating Rainmeter skin "
                "package for quick and easy distibution on Github.",
    long_description=long_description,
    long_description_content_type="text/x-rst",
    author="Brendan Doherty",
    author_email="2bndy5@gmail.com",
    install_requires=["pefile", "pillow"],
    license="MIT",
    # See https://pypi.python.org/pypi?%3Aaction=list_classifiers
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "Topic :: System :: Archiving :: Packaging",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
    ],
    keywords="rainmeter rmskin archive builder",
    py_modules=["rmskin_builder"],
    scripts=["./rmskin_builder.py"],
    # Specifiy your homepage URL for your project here
    url=REPO,
    download_url=f"{REPO}/releases",
)
