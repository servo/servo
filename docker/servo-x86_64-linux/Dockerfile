FROM ubuntu:bionic-20180821

ADD apt-install.sh /apt-install.sh
RUN /apt-install.sh \
        git \
        curl \
        ca-certificates \
        python2.7 \
        g++ \
    && \
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none -y

ENV PATH="/root/.cargo/bin:${PATH}"
