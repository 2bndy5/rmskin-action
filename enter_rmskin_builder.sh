#!/bin/sh -l

pip install -r /reqs.txt
python /rmskin_builder.py --path "$1" --version "$2" --author "$3" --title "$4" --dir_out "$5"