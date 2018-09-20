FROM ubuntu:bionic-20180821

RUN \
    apt-get update -q && \
    apt-get install -qy --no-install-recommends \
        #
        # Cloning the repository
        git \
        ca-certificates \
        #
        # Running mach
        python2.7 \
        python-virtualenv \
        virtualenv
