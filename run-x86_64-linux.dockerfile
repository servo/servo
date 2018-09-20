FROM ubuntu:bionic-20180821

ENV \
    #
    # The 'tzdata' APT package waits for user input on install by default
    # https://stackoverflow.com/questions/44331836/apt-get-install-tzdata-noninteractive
    DEBIAN_FRONTEND=noninteractive

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
        virtualenv \
        #
        # Fetching build artifacts
        curl \
        #
        # Servo’s runtime dependencies
        libgstreamer-plugins-bad1.0 \
        libssl1.0.0
