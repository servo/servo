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
# For our MSRV CI builds we need less dependencies, and smaller images mean faster builds.
# If servo fails to build, check ./mach bootstrap and investigate which additional dependency needs to be
# installed. The smaller the subset we install here, the smaller the resulting image -> faster image pulls
RUN apt-get update && \
    apt-get -y install --no-install-recommends \
        libdbus-1-dev libfreetype6-dev libgl1-mesa-dri libgles2-mesa-dev \
        libglib2.0-dev  libges-1.0-dev \
        libharfbuzz-dev liblzma-dev libudev-dev libunwind-dev \
        libvulkan1 libx11-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxmu-dev \
        libxmu6 libegl1-mesa-dev llvm-dev m4 xorg-dev libxkbcommon0 libxkbcommon-x11-0 \
        xvfb sudo

RUN mkdir -p /data/servo/servo

##### FETCH SERVO ####################################################################
FROM base_fetcher AS servo_fetcher


RUN mkdir -p /data/servo/servo

RUN cd /data/servo && git clone https://github.com/servo/servo.git
WORKDIR /data/servo/servo
# Incrementally update checked out version
ARG SERVO_COMMIT_HASH
RUN git fetch origin main && git checkout "${SERVO_COMMIT_HASH}"

##### END FETCH SERVO #################################################################


FROM base_with_cc AS rust_toolchain
# Should match the Rust MSRV!
ARG RUST_VERSION=1.85.0
ENV CARGO_HOME=/data/.cargo
ENV RUSTUP_HOME=/data/.rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain ${RUST_VERSION} \
        --profile=default


FROM base_fetcher AS uv

ARG UV_VERSION=0.8.5
RUN mkdir -p "$HOME/.local/bin" \
    && curl --proto '=https' --tlsv1.2  --fail \
        -LsSf https://github.com/astral-sh/uv/releases/download/${UV_VERSION}/uv-installer.sh | sh

FROM base AS servo_base
COPY --from=rust_toolchain /data/.cargo /data/.cargo
COPY --from=rust_toolchain /data/.rustup /data/.rustup
ENV CARGO_HOME=/data/.cargo
ENV RUSTUP_HOME=/data/.rustup
ENV PATH="/data/.cargo/bin:${PATH}"
COPY --from=uv /root/.local/bin/uv /root/.local/bin/uvx /data/.cargo/bin/

##### Pre-run cargo check ####################################################################
FROM servo_base AS servo_check_cook

COPY --from=servo_fetcher /data/servo/servo /data/servo/servo
WORKDIR /data/servo/servo
ARG RUST_VERSION=1.85.0
RUN cargo +${RUST_VERSION} fetch --target x86_64-unknown-linux-gnu
# Run cargo check twice, so we can use incremental cargo check in CI.
# Note: twice is necessary to avoid rebuilding due to generated files
RUN mkdir -p /data/servo/target \
    && cargo +${RUST_VERSION} check --locked -p libservo --target-dir /data/servo/target \
    && cargo +${RUST_VERSION} check --locked -p libservo --target-dir /data/servo/target


#FROM servo_base AS final
## Add the target directory to the image, so we can do incremental checks in CI.
## Trade-off image size (longer image download) for faster / incremental cargo check in CI
#COPY --from=servo_check_cook /data/servo/target /data/servo/target
