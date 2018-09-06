FROM ubuntu:bionic-20180821

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        git \
        curl \
        ca-certificates \
        python2.7 \
        g++ \
    && \
    rm -rf /var/lib/apt/lists/* && \
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y

ENV PATH="/root/.cargo/bin:${PATH}"
