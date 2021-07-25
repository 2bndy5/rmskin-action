#! /usr/bin/python3
"""
A script that will attempt to assemble a validating Rainmeter skin package for
quick and easy distibution on Github.

Ideal Repo Structure
********************

- root directory

    * ``Skins``      <- a folder to contain all necessary Rainmeter skins
    * ``RMSKIN.ini`` <- list of options specific to installing the skin(s)
    * ``Layouts``    <- a folder that contains Rainmeter layout files
    * ``Plugins``    <- a folder that contains Rainmeter plugins
    * ``@Vault``     <- resources folder accessible by all installed skins

.. seealso::
    `A cookiecutter repository <https://github.com/2bndy5/Rainmeter-Cookiecutter>`_
    has also been created to facilitate development of Rainmeter skins on Github
    quickly.
"""
import os
import argparse
import configparser
import zipfile
import struct
import logging
import pefile
from PIL import Image


parser = argparse.ArgumentParser(
    description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
)
parser.add_argument(
    "--path",
    metavar='"str"',
    type=str,
    default=os.getenv("GITHUB_WORKSPACE", os.getcwd()),
    help="Base path of a git repository. Defaults to working directory.",
)
parser.add_argument(
    "--version",
    metavar='"str"',
    type=str,
    default="auto",
    help="Version of release. This should be the github action env var "
    "GITHUB_REF ('refs/tags') or last 8 digits of GITHUB_SHA.",
)
parser.add_argument(
    "--author",
    metavar='"str"',
    type=str,
    default="Unknown",
    help="Author of release. This should be the github action env var GITHUB_ACTOR.",
)
parser.add_argument(
    "--title",
    metavar='"str"',
    type=str,
    default=os.path.split(os.getcwd())[1],
    help="Title of released package. This should be just the github repo name.",
)
parser.add_argument(
    "--dir_out",
    metavar='"str"',
    type=str,
    default=None,
    help="Output path to save released package file. This optional & only used when specified.",
)

# setup logging output
logging.basicConfig()
LOGGER_NAME = os.path.split(__file__)[1].split(".")[0].split("_")
LOGGER_NAME[0] = LOGGER_NAME[0].upper()
LOGGER_NAME[1] = LOGGER_NAME[1].title()
LOGGER_NAME = " ".join(LOGGER_NAME)
logger = logging.getLogger(LOGGER_NAME)
logger.setLevel(logging.INFO)

#: The `dict` of package components discovered by the `main()` loop
HAS_COMPONENTS = {
    "RMSKIN.ini": False,
    "Skins": 0,
    "Layouts": 0,
    "Plugins": False,
    "@Vault": 0,
    "RMSKIN.bmp": False,
}


def discover_components():
    """The method that does priliminary dscovery of rmskin package components."""
    for dirpath, dirnames, filenames in os.walk(root_path):
        dirpath = dirpath.replace(root_path, "")
        if dirpath.endswith("Skins"):
            HAS_COMPONENTS["Skins"] = len(dirnames)
            logger.info("Found %d possible Skin(s)", HAS_COMPONENTS["Skins"])
        elif dirpath.endswith("@Vault"):
            HAS_COMPONENTS["@Vault"] = len(filenames) + len(dirnames)
            logger.info("Found %d possible @Vault item(s)", HAS_COMPONENTS["@Vault"])
        elif dirpath.endswith("Plugins"):
            if len(dirnames) > 0:
                HAS_COMPONENTS["Plugins"] = True
            logger.info("Found Plugins folder")
        elif dirpath.endswith("Layouts"):
            HAS_COMPONENTS["Layouts"] = len(filenames) + len(dirnames)
            logger.info("Found %d possible Layout(s)", HAS_COMPONENTS["Layouts"])
        elif len(dirpath) == 0:
            if "RMSKIN.ini" in filenames:
                HAS_COMPONENTS["RMSKIN.ini"] = True
                logger.info("Found RMSKIN.ini file")
            if "RMSKIN.bmp" in filenames:
                HAS_COMPONENTS["RMSKIN.bmp"] = True
                logger.info("Found header image")
            for dir_name in dirnames:  # exclude hidden directories
                if dir_name.startswith("."):
                    del dir_name
        # set depth of search to shallow (2 folders deep)
        if len(dirpath) > 0:
            dirnames.clear()


def parse_rmskin_ini():
    """Read the RMSKIN.ini and write a copy for building the RMSKIN package."""
    arc_name = args.title
    version = args.version
    config = configparser.ConfigParser()

    config.read(root_path + os.sep + "RMSKIN.ini")
    if "rmskin" in config:
        if "Version" in config["rmskin"]:
            version = config["rmskin"]["Version"]
        if version.endswith("auto"):
            if not os.getenv("GITHUB_REF", "").startswith("refs/tags/"):
                version = os.getenv("GITHUB_SHA", "x0x.x0xy")[-8:]
            else:
                version = os.getenv("GITHUB_REF", "refs/tags/0.0").replace(
                    "refs/tags/", ""
                )
            config["rmskin"]["Version"] = version
        if not "Author" in config["rmskin"]:
            # maybe someday, aggregate list of authors from
            # discovered skins' metadata->Author fields
            config["rmskin"]["Author"] = args.author
        if "Name" in config["rmskin"]:
            # use hard-coded name
            arc_name = config["rmskin"]["Name"]
        else:
            # use repo name
            config["rmskin"]["Name"] = args.title
        logger.info("Using Name (%s) & Version (%s)", arc_name, version)
        load_t = config["rmskin"]["LoadType"]  # ex: "Skin"
        load = config["rmskin"]["Load"]  # ex: "Skin_Root\\skin.ini"
        # for cross-platform compatibility, adjust windows-style path seperators
        load = load.replace("\\", os.sep)
        if len(load_t):  # if a file set to load on-install
            # exit early if loaded file does not exist
            temp = (
                root_path
                + os.sep
                + load_t
                + "s"
                + os.sep
                + (load if load_t == "Skin" else load + os.sep + "Rainmeter.ini")
            )
            if not os.path.isfile(temp):
                raise RuntimeError("On-install loaded file does not exits.")
    else:
        raise RuntimeError("RMSKIN.ini is malformed")
    with open(BUILD_DIR + "RMSKIN.ini", "w") as conf:
        config.write(conf)  # Dump changes/corrections to temp build dir
    return (arc_name, version)


def validate_header_image():
    """Make sure header image (if any) is ready to package"""
    if HAS_COMPONENTS["RMSKIN.bmp"]:
        with Image.open(root_path + os.sep + "RMSKIN.bmp") as img:
            if img.width != 400 and img.height != 60:
                logger.warning("Resizing header image to 400x60")
                img = img.resize((400, 60))
            if img.mode != "RGB":
                logger.warning("Correcting color space in header image.")
                img = img.convert(mode="RGB")
            img.save(BUILD_DIR + "RMSKIN.bmp")


def init_zip_for_package(arch_name):
    """Create initial archive to use as RMSKIN package"""
    # pylint: disable=too-many-nested-blocks
    output_path_to_archive = (
        (root_path if args.dir_out is None else args.dir_out) + os.sep + arch_name
    )
    with zipfile.ZipFile(
        output_path_to_archive,
        "w",
        compression=zipfile.ZIP_DEFLATED,
        compresslevel=9,
    ) as arc_file:
        # write RMSKIN.ini and header image (RMSKIN.bmp) first
        if HAS_COMPONENTS["RMSKIN.bmp"]:
            arc_file.write(BUILD_DIR + "RMSKIN.bmp", arcname="RMSKIN.bmp")
        arc_file.write(BUILD_DIR + "RMSKIN.ini", arcname="RMSKIN.ini")
        for key, val in HAS_COMPONENTS.items():
            if not key.endswith(".ini") and val:
                for dirpath, _, filenames in os.walk(root_path + os.sep + key):
                    if key.endswith("Plugins"):
                        # check bitness of plugins here & archive accordingly
                        for file_name in filenames:
                            if file_name.lower().endswith(".dll"):
                                # let plugin_name be 2nd last folder name in dll's path
                                bitness = pefile.PE(
                                    dirpath + os.sep + file_name,
                                    fast_load=True,  # just get headers
                                )
                                bitness.close()  # do this now to copy file safely later
                                # pylint: disable=no-member
                                if bitness.FILE_HEADER.Machine == 0x014C:
                                    # archive this 32-bit plugin
                                    arc_file.write(
                                        dirpath + os.sep + file_name,
                                        arcname=key
                                        + os.sep
                                        + "32bit"
                                        + os.sep
                                        + file_name,
                                    )
                                # pylint: enable=no-member
                                else:
                                    # archive this 64-bit plugin
                                    arc_file.write(
                                        dirpath + os.sep + file_name,
                                        arcname=key
                                        + os.sep
                                        + "64bit"
                                        + os.sep
                                        + file_name,
                                    )
                                del bitness
                            else:  # for misc files in plugins folders like READMEs
                                arc_file.write(
                                    dirpath + os.sep + file_name,
                                    arcname=dirpath.replace(root_path + os.sep, "")
                                    + os.sep
                                    + file_name,
                                )
                    else:
                        for file_name in filenames:
                            arc_file.write(
                                dirpath + os.sep + file_name,
                                arcname=dirpath.replace(root_path + os.sep, "")
                                + os.sep
                                + file_name,
                            )
        # archive assembled; closing file
    # pylint: enable=too-many-nested-blocks
    return output_path_to_archive

def main():
    """The main execution loop for creating a rmskin package."""
    logger.info("Searching path: %s", root_path)

    # capture the directory tree
    discover_components()

    # quit if bad dir struct
    if not (
        HAS_COMPONENTS["Layouts"]
        or HAS_COMPONENTS["Skins"]
        or HAS_COMPONENTS["Plugins"]
        or HAS_COMPONENTS["@Vault"]
    ):
        raise RuntimeError(
            f"repository structure for {root_path} is malformed. Found no Skins,"
            " Layouts, Plugins, or @Vault assets."
        )

    # quit if no RMSKIN.ini
    if not HAS_COMPONENTS["RMSKIN.ini"]:
        raise RuntimeError(
            f"repository structure for {root_path} is malformed. RMSKIN.ini file "
            "not found."
        )

    # read options from RMSKIN.ini
    arc_name, version = parse_rmskin_ini()

    # make sure header image is correct size (400x60) & correct color space
    validate_header_image()

    # Now creating the archive
    archive_name = arc_name + "_" + version + ".rmskin"
    path_to_archive = init_zip_for_package(archive_name)

    compressed_size = 0
    compressed_size = os.path.getsize(path_to_archive)
    logger.info("archive size = %d (0x%X)", compressed_size, compressed_size)

    # convert size to a bytes obj & prepend to custom footer
    custom_footer = struct.pack("q", compressed_size) + b"\x00RMSKIN\x00"

    # append footer to archive
    with open(path_to_archive, "a+b") as arc_file:
        logger.debug("appending footer: %s", repr(custom_footer))
        arc_file.write(custom_footer)
    logger.info("Archive successfully prepared.")

    # env var CI is always true when executed on a github action runner
    if os.getenv("CI", "false").title().startswith("True"):
        print("::set-output name=arc_name::{}".format(archive_name))
    else:
        logger.info("archive name: %s", archive_name)


if __name__ == "__main__":
    # collect cmd args
    args = parser.parse_args()
    root_path = args.path
    # truncate trailing path seperator
    if root_path.endswith(os.sep):
        root_path = root_path[:-1]
    root_path = os.path.abspath(root_path)

    # The temporary build dir for storing altered files
    BUILD_DIR = root_path + os.sep + "build" + os.sep
    if not os.path.isdir(BUILD_DIR):
        os.mkdir(BUILD_DIR)

    if args.dir_out is not None and args.dir_out.endswith(os.sep):
        args.dir_out = args.dir_out.rstrip(os.sep)
    main()
