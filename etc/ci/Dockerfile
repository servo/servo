FROM debian:jessie
MAINTAINER Servo Dockerfile Maintainers <servo-dockerfile-maintainers@mozilla.com>

# Rust installation from https://hub.docker.com/r/jimmycuadra/rust/
ENV USER root
ENV RUST_VERSION=1.18.0

RUN apt-get update && \
  DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    git \
    libssl-dev \
    pkg-config && \
  curl -sO https://static.rust-lang.org/dist/rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz && \
  tar -xzf rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz && \
  ./rust-$RUST_VERSION-x86_64-unknown-linux-gnu/install.sh --without=rust-docs && \
  DEBIAN_FRONTEND=noninteractive apt-get remove --purge -y curl && \
  DEBIAN_FRONTEND=noninteractive apt-get autoremove -y && \
  rm -rf \
    rust-$RUST_VERSION-x86_64-unknown-linux-gnu \
    rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz \
    /var/lib/apt/lists/* \
    /tmp/* \
    /var/tmp/* && \
  mkdir /source

# Servo's requirements
RUN apt-get update && apt-get install -y \
  xvfb \
  python \
  python3 \
  python-pip

RUN pip install virtualenv
RUN useradd servo

# TODO: RUN wget https://raw.githubusercontent.com/servo/saltfs/master/xvfb/xvfb.conf /etc/init/xvfb.conf
# TODO: cross: Servo's servo-build-depenencies salt state
# TODO: cross: Servo's servo-build-dependencies.android
# TODO: cross: Servo's servo-build-dependencies.arm
# TODO: builder-specific config from saltfs/buildbot/master/files/config/environments.py

