% include run.dockerfile

RUN apt-get install -qy --no-install-recommends \
    python3 \
    jq
