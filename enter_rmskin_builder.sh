#!/bin/sh -l

pip install -r $GITHUB_WORKSPACE/reqs.txt
python $GITHUB_WORKSPACE/rmskin_builder.py