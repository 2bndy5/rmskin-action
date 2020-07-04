# Container image that runs your code
FROM python

# Copies your code file from your action repository to the filesystem path `/` of the container
COPY enter_rmskin_builder.sh /enter_rmskin_builder.sh
COPY release.py /rmskin_builder.py
COPY requirements.txt /reqs.txt

# Code file to execute when the docker container starts up (`enter_rmskin_builder.sh`)
ENTRYPOINT ["/enter_rmskin_builder.sh"]