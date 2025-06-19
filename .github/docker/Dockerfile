# Dockerfile to build servo in CI with the Rust MSRV
# We use multi-stage builds to reduce the size of the final image and
# allow parallel building of independant layers

FROM ubuntu:22.04 AS base_fetcher

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get -y install --no-install-recommends \
    ca-certificates unzip curl git jq \
    && apt-get clean


FROM base_fetcher AS base_with_cc
# We need a c-compiler to `cargo install` some of our dependencies
RUN apt-get update && \
    apt-get -y install --no-install-recommends \
    build-essential ccache clang cmake python3 \
    && apt-get clean

FROM base_with_cc AS base
# Install the remaining dependencies in a new stage, so that `cargo install` can run in parallel

# gstreamer1.0-plugins-good libgstreamer-plugins-good1.0-dev \
  #        gstreamer1.0-plugins-bad libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-ugly \
  #        gstreamer1.0-plugins-base libgstreamer-plugins-base1.0-dev gstreamer1.0-libav \
  #        libgstrtspserver-1.0-dev gstreamer1.0-tools

RUN apt-get update && \
    apt-get -y install --no-install-recommends \
        libdbus-1-dev libfreetype6-dev libgl1-mesa-dri libgles2-mesa-dev \
        libglib2.0-dev  libges-1.0-dev \
        libharfbuzz-dev liblzma-dev libudev-dev libunwind-dev \
        libvulkan1 libx11-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxmu-dev \
        libxmu6 libegl1-mesa-dev llvm-dev m4 xorg-dev libxkbcommon0 libxkbcommon-x11-0 \
        xvfb sudo

RUN mkdir -p /data/servo/servo

FROM base_with_cc AS rust_toolchain
# Should match the Rust MSRV!
ARG RUST_VERSION=1.85.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain ${RUST_VERSION} \
        --profile=minimal

##### FETCH SERVO ####################################################################
FROM base_fetcher AS servo_fetcher


RUN mkdir -p /data/servo/servo
# Clone servo, ignoring the history.
RUN cd /data/servo && git clone https://github.com/servo/servo.git --depth 1
WORKDIR /data/servo/servo

##### END FETCH SERVO #################################################################

FROM base_fetcher AS uv

ARG UV_VERSION=0.7.9
RUN mkdir -p "$HOME/.local/bin" \
    && curl --proto '=https' --tlsv1.2  --fail \
        -LsSf https://github.com/astral-sh/uv/releases/download/${UV_VERSION}/uv-installer.sh | sh

FROM base AS servo_base

WORKDIR /data/servo/servo

COPY --from=rust_toolchain /root/.cargo /root/.cargo
COPY --from=rust_toolchain /root/.rustup /root/.rustup
COPY --from=uv /root/.local/bin/uv /root/.local/bin/uvx /root/.cargo/bin/
ENV PATH="/root/.cargo/bin:${PATH}"

COPY --from=servo_fetcher /data/servo /data/servo

# ideally this should be a no-op: Todo: investigate the layer and see if it is.
RUN ./mach bootstrap --skip-platform

# Create a layer which prefetches git dependencies
RUN cargo +1.85.0 update -p mozjs && git checkout Cargo.lock
