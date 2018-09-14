FROM ubuntu:bionic-20180821

ENV \
    #
    # Use rustup’s 'cargo' and 'rustc'
    PATH="/root/.cargo/bin:${PATH}" \
    #
    # SpiderMonkey’s build system fails if $SHELL is unset
    SHELL=/bin/dash \
    #
    # The 'tzdata' APT package waits for user input on install by default
    # https://stackoverflow.com/questions/44331836/apt-get-install-tzdata-noninteractive
    DEBIAN_FRONTEND=noninteractive

RUN apt-get update -q && apt-get install -qy --no-install-recommends \
    #
    # Cloning the repository
    git \
    ca-certificates \
    #
    # Installing rustup
    curl \
    #
    # Running mach
    python2.7 \
    python-virtualenv \
    virtualenv \
    #
    # Multiple C/C++ dependencies built from source
    g++ \
    make \
    cmake \
    #
    # ANGLE
    xorg-dev \
    #
    # mozjs (SpiderMonkey)
    autoconf2.13 \
    #
    # Bindgen (for SpiderMonkey bindings)
    clang \
    #
    # gstreamer
    libglib2.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libgstreamer-plugins-bad1.0-dev \
    libgstreamer1.0-dev \
    #
    # OpenSSL
    libssl1.0-dev \
    #
    # blurz
    libdbus-1-dev \
    #
    # Skia
    libglu1-mesa-dev \
    libbz2-dev \
    #
    #
    && \
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y

