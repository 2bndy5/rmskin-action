"""
A script to run on github release action that will
attempt to assemble a validating Rainmeter skin
package for quick and easy distibution.

ideal repo structure
********************

    - root directory
        * ``Skins`` (a folder to contain all necessary skins)
        * ``RMSKIN.ini`` (list of options specific to installing the
          skin).
        * ``Layouts``(a folder that contains rainmeter layout files)
        * ``Plugins``(a folder that contains rainmeter plugins)
        * ``@Vault`` (resources folder accessible by all installed
          skins)
"""
import os
import argparse
import configparser
import zipfile
import struct
import pefile
from PIL import Image

parser = argparse.ArgumentParser(
    description="""
    A script that will attempt to assemble a 
    validating Rainmeter skin package for 
    quick and easy githuib distibution."""
)
parser.add_argument(
    "--path",
    metavar='"STR"',
    type=str,
    default=os.getenv("GITHUB_WORKSPACE", os.getcwd()),
    help="Base path of a git repository. Defaults to working directory.",
)
parser.add_argument(
    "--version",
    metavar='"STR"',
    type=str,
    default='auto',
    help="Version of release. This should be the github action env var (GITHUB_REF - 'refs/tags' or last 8 digits of GITHUB_SHA).",
)
parser.add_argument(
    "--author",
    metavar='"STR"',
    type=str,
    default="Unknown",
    help="Author of release. This should be the github action env var (GITHUB_ACTOR).",
)
parser.add_argument(
    "--title",
    metavar='"STR"',
    type=str,
    default=os.getcwd().split(os.sep)[len(os.getcwd().split(os.sep)) - 1],
    help="Title of released package. This should be just the github repo name.",
)
parser.add_argument(
    "--dir_out",
    metavar='"STR"',
    type=str,
    default=None,
    help="Output path to save released package file. This optional & only used when specified.",
)

HAS_COMPONENTS = {
    "RMSKIN.ini": False,
    "Skins": 0,
    "Layouts": 0,
    "Plugins": False,
    "@Vault": 0,
    "RMSKIN.bmp": False
}


def main():
    # collect cmd args
    args = parser.parse_args()
    root_path = args.path
    # truncate trailing path seperator
    if root_path.endswith(os.sep):
        root_path = root_path[:-1]
    if args.dir_out is not None and args.dir_out.endswith(os.sep):
        args.dir_out = args.dir_out[:-1]
    print(f"using path: {root_path}")

    # capture the directory tree
    for dirpath, dirnames, filenames in os.walk(root_path):
        dirpath = dirpath.replace(root_path, "")
        if dirpath.endswith("Skins"):
            HAS_COMPONENTS["Skins"] = len(dirnames)
            print(f"Found {HAS_COMPONENTS['Skins']} possible Skin(s)")
        elif dirpath.endswith("@Vault"):
            HAS_COMPONENTS["@Vault"] = len(filenames) + len(dirnames)
            print(f"Found {HAS_COMPONENTS['@Vault']} possible @Vault item(s)")
        elif dirpath.endswith("Plugins"):
            if len(dirnames) > 0:
                HAS_COMPONENTS["Plugins"] = True
            print("Found Plugins folder")
        elif dirpath.endswith("Layouts"):
            HAS_COMPONENTS["Layouts"] = len(filenames) + len(dirnames)
            print(f"Found {HAS_COMPONENTS['Layouts']} possible Layout(s)")
        elif len(dirpath) == 0:
            if "RMSKIN.ini" in filenames:
                HAS_COMPONENTS["RMSKIN.ini"] = True
                print("Found RMSKIN.ini file")
            if "RMSKIN.bmp" in filenames:
                HAS_COMPONENTS["RMSKIN.bmp"] = True
                print("Found header image")
            for d in dirnames:  # exclude hidden directories
                if d.startswith("."):
                    del d
        # set depth of search to shallow (2 folders deep)
        if len(dirpath) > 0:
            dirnames.clear()
    
    # quite if bad dir struct
    if not (
        HAS_COMPONENTS["Layouts"]
        or HAS_COMPONENTS["Skins"]
        or HAS_COMPONENTS["Plugins"]
        or HAS_COMPONENTS["@Vault"]
    ):
        raise RuntimeError(
            f"repository structure for {root_path} is malformed. Found no Skins,"
            " Layouts, or Plugins!"
        )
    # read options from RMSKIN.ini
    arc_name = args.title
    version = "auto"
    config = configparser.ConfigParser()
    if HAS_COMPONENTS["RMSKIN.ini"]:
        config.read(root_path + os.sep + "RMSKIN.ini")
        if "rmskin" in config:
            if "Version" in config["rmskin"]:
                version = config["rmskin"]["Version"]
            if version.endswith("auto"):
                if not os.getenv("GITHUB_REF", "").startswith("refs/tags/"):
                    version = os.getenv("GITHUB_SHA", "x0x.x0xy")[-8:]
                else:
                    version = os.getenv("GITHUB_REF", "refs/tags/0.0").replace("refs/tags/", "")
                config["rmskin"]["Version"] = version
            if not "Author" in config["rmskin"]:
                # maybe someday aggregated list authors from discovered skins' metadata->Author fields
                config["rmskin"]["Author"] = args.author
            if "Name" in config["rmskin"]:
                # use hard-coded name
                arc_name = config["rmskin"]["Name"]
            else:
                # use repo name
                config["rmskin"]["Name"] = args.title
            print(f"Using Name ({arc_name}) & Version ({version})")
            load_t = config["rmskin"]["LoadType"]  # ex: "Skin"
            load = config["rmskin"]["Load"]  # ex: "Skin_Root\\skin.ini"
            # for cross-platform compatibility, adjust windows-style path seperators
            load = load.replace("\\", os.sep)
            if len(load_t):  # if a file set to load on-install
                # exit early if loaded file does not exist
                with open(
                    root_path + os.sep + load_t + "s" + os.sep + (load if load_t == "Skin" else load + os.sep + "Rainmeter.ini"), "r"
                ) as temp:
                    if temp is None:
                        raise RuntimeError("On-install loaded file does not exits.")
        else:
            raise RuntimeError("RMSKIN.ini is malformed")
        with open(root_path + os.sep + "RMSKIN.ini", "w") as conf:
            config.write(conf)  # Dump changes/corrections back2file
    else:
        raise RuntimeError(
            f"repository structure for {root_path} is malformed. RMSKIN.ini file not found!"
        )
    
    # make sure header image is correct size (400x60) & correct color space
    if HAS_COMPONENTS["RMSKIN.bmp"]:
        with Image.open(root_path + os.sep + "RMSKIN.bmp") as img:
            if img.width != 400 and  img.height != 60:
                print("WARNING: resizing header image to 400x60")
                img = img.resize((400, 60))
            if img.mode != 'RGB':
                print("Correcting color space in header image.")
                img = img.convert(mode='RGB')
            img.save(root_path + os.sep + "RMSKIN.bmp")

    # Now creating the archive
    compressed_size = 0
    with zipfile.ZipFile(
        (root_path if args.dir_out is None else args.dir_out)
        + os.sep + arc_name + "_" + version + ".rmskin",
        "w",
        compression=zipfile.ZIP_DEFLATED,
        compresslevel=9,
    ) as arc_file:
        # write RMSKIN.ini and header image (RMSKIN.bmp) first
        if HAS_COMPONENTS["RMSKIN.bmp"]:
            arc_file.write(root_path + os.sep + "RMSKIN.bmp", arcname="RMSKIN.bmp")
        arc_file.write(root_path + os.sep + "RMSKIN.ini", arcname="RMSKIN.ini")
        for key in HAS_COMPONENTS:
            if key.endswith(".ini"):
                pass
            elif HAS_COMPONENTS[key]:
                for dirpath, dirnames, filenames in os.walk(root_path + os.sep + key):
                    if key.endswith("Plugins"):
                        # check bitness of plugins here & archive accordingly
                        for n in filenames:
                            if n.lower().endswith(".dll"):
                                # let plugin_name be 2nd last folder name in dll's path
                                bitness = pefile.PE(
                                    dirpath + os.sep + n,
                                    fast_load=True,  # just get headers
                                )
                                bitness.close()  # do this now to copy file safely later
                                # pylint: disable=no-member
                                if bitness.FILE_HEADER.Machine == 0x014C:
                                    # archive this 32-bit plugin
                                    arc_file.write(
                                        dirpath + os.sep + n,
                                        arcname=key + os.sep + "32bit" + os.sep + n,
                                    )
                                else:
                                    # archive this 64-bit plugin
                                    arc_file.write(
                                        dirpath + os.sep + n,
                                        arcname=key + os.sep + "64bit" + os.sep + n,
                                    )
                                # pylint: enable=no-member
                                del bitness
                            else:  # for misc files in plugins folders like READMEs
                                arc_file.write(
                                    dirpath + os.sep + n,
                                    arcname=dirpath.replace(root_path + os.sep, "")
                                    + os.sep
                                    + n,
                                )
                    else:
                        for n in filenames:
                            arc_file.write(
                                dirpath + os.sep + n,
                                arcname=dirpath.replace(root_path + os.sep, "")
                                + os.sep
                                + n,
                            )
        # archive assembled; closing file
    compressed_size = os.path.getsize(arc_name + "_" + version + ".rmskin")
    print(f"archive size = {compressed_size} ({hex(compressed_size)})")
    # convert size to a bytes obj & prepend to custom footer
    custom_footer = struct.pack("q", compressed_size) + b"\x00RMSKIN\x00"

    # append footer to archive
    with open(arc_name + "_" + version + ".rmskin", "a+b") as arc_file:
        print(f"appending footer: {custom_footer}")
        arc_file.write(custom_footer)

    print("Archive successfully prepared.")
    print("::set-output name=arc_name::{}".format(arc_name + "_" + version + ".rmskin"))


if __name__ == "__main__":
    main()
