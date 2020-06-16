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
        #
    && \
    #
    # Install the version of rustup that is current when this Docker image is being built:
    # We want at least 1.21 (increment in this comment to force an image rebuild).
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none --profile=minimal -y && \
    #
    #
    curl -sSfL \
        https://github.com/mozilla/sccache/releases/download/0.2.12/sccache-0.2.12-x86_64-unknown-linux-musl.tar.gz \
        | tar -xz --strip-components=1 -C /usr/local/bin/ \
            sccache-0.2.12-x86_64-unknown-linux-musl/sccache
