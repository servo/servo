FROM ubuntu:eoan-20200608

ENV \
    #
    # Some APT packages like 'tzdata' wait for user input on install by default.
    # https://stackoverflow.com/questions/44331836/apt-get-install-tzdata-noninteractive
    DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    LANGUAGE=C.UTF-8 \
    LC_ALL=C.UTF-8

RUN \
    apt-get update -q && \
    apt-get install -qy --no-install-recommends \
        #
        # Cloning the repository
        git \
        ca-certificates \
        #
        # Running mach with Python 2
        python \
        python-pip \
        python-dev \
        python-virtualenv \
        #
        # Running mach with Python 3
        python3 \
        python3-pip \
        python3-dev \
        virtualenv \
        #
        # Compiling C modules when installing Python packages in a virtualenv
        gcc \
        #
        # Installing rustup and sccache (build dockerfile) or fetching build artifacts (run tasks)
        curl \
        # Setting the default locale
        locales \
        locales-all

