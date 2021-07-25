# Container image that runs your code
FROM python:latest

# Copies your code file from your action repository to the filesystem path `/` of the container
COPY enter_rmskin_builder.sh /enter_rmskin_builder.sh
COPY rmskin_builder.py /rmskin_builder.py
COPY requirements.txt /reqs.txt
RUN chmod +x /enter_rmskin_builder.sh

# Code file to execute when the docker container starts up (`enter_rmskin_builder.sh`)
ENTRYPOINT ["/enter_rmskin_builder.sh"]
