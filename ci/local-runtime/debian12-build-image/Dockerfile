# Debian 12/bookworm Servo build-room image for local-runtime artifacts.
#
# This image intentionally contains only reusable build prerequisites. It does
# not copy Servo source, does not run ./mach bootstrap, and does not contain a
# Cargo target directory or compiled Servo artifacts.
FROM debian:12

ENV DEBIAN_FRONTEND=noninteractive \
    SHELL=/bin/bash \
    CC=clang \
    CXX=clang++ \
    CARGO_TARGET_DIR=/var/servo-cargo-target \
    UV_PROJECT_ENVIRONMENT=.devcontainer-venv \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        binutils \
        build-essential \
        ca-certificates \
        ccache \
        clang \
        cmake \
        curl \
        g++ \
        git \
        gperf \
        gstreamer1.0-libav \
        gstreamer1.0-plugins-bad \
        gstreamer1.0-plugins-base \
        gstreamer1.0-plugins-good \
        gstreamer1.0-plugins-ugly \
        gstreamer1.0-tools \
        libdbus-1-dev \
        libegl1-mesa-dev \
        libfreetype6-dev \
        libges-1.0-dev \
        libgles2-mesa-dev \
        libglib2.0-dev \
        libgl1-mesa-dri \
        libgstreamer-plugins-bad1.0-dev \
        libgstreamer-plugins-base1.0-dev \
        libgstrtspserver-1.0-dev \
        libharfbuzz-dev \
        liblzma-dev \
        libudev-dev \
        libunwind-dev \
        libvulkan1 \
        libx11-dev \
        libxcb-render0-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        libxkbcommon0 \
        libxkbcommon-x11-0 \
        libxmu-dev \
        libxmu6 \
        llvm-dev \
        m4 \
        ninja-build \
        pkg-config \
        python3 \
        tar \
        xorg-dev \
        zip \
    && rm -rf /var/lib/apt/lists/*

# Servo's ./mach re-execs through uv before Python bootstrap can install any
# additional dependencies. Install uv into a fixed system location so container
# jobs can use it without sourcing shell profiles.
RUN curl -LsSf https://astral.sh/uv/install.sh | UV_INSTALL_DIR=/usr/local/bin sh \
    && uv --version

# The plain Debian image does not include Rust. Servo bootstrap installs Cargo
# tools before/around the broader dependency setup, so provide rustup/cargo/rustc
# in the same system locations used by the Servo devcontainer family. The final
# toolchain will still be governed by the checkout's rust-toolchain.toml.
RUN curl https://sh.rustup.rs -sSf \
        | sh -s -- --default-toolchain stable -y --component clippy,llvm-tools,llvm-tools-preview,rustc-dev,rustfmt,rust-src \
    && chmod -R a+w "$RUSTUP_HOME" "$CARGO_HOME" \
    && rustup --version \
    && rustc --version \
    && cargo --version

RUN mkdir -p "$CARGO_TARGET_DIR" \
    && chmod -R a+w "$CARGO_TARGET_DIR"

CMD ["/bin/bash"]
