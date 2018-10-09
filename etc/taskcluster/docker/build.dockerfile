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
        # ANGLE
        xorg-dev \
        #
        # mozjs (SpiderMonkey)
        autoconf2.13 \
        #
        # Bindgen (for SpiderMonkey bindings)
        clang \
        #
        # GStreamer
        libgstreamer-plugins-bad1.0-dev \
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
    #
    #
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y && \
    #
    #
    curl -sSfL \
        https://github.com/mozilla/sccache/releases/download/0.2.7/sccache-0.2.7-x86_64-unknown-linux-musl.tar.gz \
        | tar -xz --strip-components=1 -C /usr/local/bin/ \
            sccache-0.2.7-x86_64-unknown-linux-musl/sccache
