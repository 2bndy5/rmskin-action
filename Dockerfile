# Container image that runs your code
FROM python

# Copies your code file from your action repository to the filesystem path `/` of the container
COPY enter_rmskin_builder.sh $GITHUB_WORKSPACE/enter_rmskin_builder.sh
COPY release.py $GITHUB_WORKSPACE/rmskin_builder.py
COPY requirements.txt $GITHUB_WORKSPACE/reqs.txt
RUN chmod +x $GITHUB_WORKSPACE/enter_rmskin_builder.sh
# RUN ls

# Code file to execute when the docker container starts up (`enter_rmskin_builder.sh`)
ENTRYPOINT ["/enter_rmskin_builder.sh"]