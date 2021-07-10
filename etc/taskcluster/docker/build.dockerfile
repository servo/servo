% include base.dockerfile

RUN \
    apt-get install -qy --no-install-recommends \
        #
        # Testing decisionlib (see etc/taskcluster/mock.py)
        python3-coverage \
        #
        # Multiple C/C++ dependencies built from source
        g++ \
        make \
        cmake \
        #
        # Fontconfig
        gperf \
        #
        # ANGLE
        xorg-dev \
        #
        # mozjs (SpiderMonkey)
        autoconf2.13 \
        #
        # Bindgen (for SpiderMonkey bindings)
        clang \
        llvm \
        llvm-dev \
        #
        # GStreamer
        libpcre3-dev \
        #
        # OpenSSL
        libssl-dev \
        #
        # blurz
        libdbus-1-dev \
        #
        # sampling profiler
        libunwind-dev \
        #
        # x11 integration
        libxcb-render-util0-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        #
    && \
    #
    # Install the version of rustup that is current when this Docker image is being built:
    # We want at least 1.21 (increment in this comment to force an image rebuild).
    curl https://sh.rustup.rs -sSf | sh -s -- --profile=minimal -y && \
    #
    # There are no sccache binary releases that include this commit, so we install a particular
    # git commit instead.
    ~/.cargo/bin/cargo install sccache --git https://github.com/mozilla/sccache/ --rev e66c9c15142a7e583d6ab80bd614bdffb2ebcc47
