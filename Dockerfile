FROM ubuntu:xenial
LABEL maintainer="servo-dockerfile-maintainers@mozilla.com"

# Note: This Dockerfile sets up the test environment for running Servo. It
# does NOT actually perform any build steps by default -- the build steps are
# listed in etc/ci/buildbot_steps.yml.

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

# Servo's apt packages & Python salt state
RUN apt-get update && apt-get install -y \
  xvfb \
  python \
  python3 \
  python-pip

RUN pip install virtualenv

# Servo's common salt state

RUN useradd servo

# TODO: cross: Servo's servo-build-depenencies salt state
# TODO: cross: Servo's servo-build-dependencies.android
# TODO: cross: Servo's servo-build-dependencies.arm
# TODO: Servo's xvfb state
# RUN wget https://raw.githubusercontent.com/servo/saltfs/master/xvfb/xvfb.conf /etc/init/xvfb.conf
# TODO: the builder-specific bits from saltfs/buildbot/master/files/config/environments.py

# etc/ci/buildbot_steps.yml, as hardcoded in salt factories.py, contains
# steps to run


