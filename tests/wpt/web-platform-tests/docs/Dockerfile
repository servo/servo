FROM ubuntu:20.04

# No interactive frontend during docker build
ENV DEBIAN_FRONTEND=noninteractive \
    DEBCONF_NONINTERACTIVE_SEEN=true

# General requirements not in the base image
RUN apt-get -qqy update \
  && apt-get -qqy install \
    ca-certificates \
    curl \
    git \
    npm \
    python3 \
    python3-distutils \
    python3-pip \
    python3.9 \
    python3-distutils \
    python3.9-venv \
    software-properties-common \
    tzdata \
    sudo \
    unzip \
  # Set Python 3.9 as the default
  && update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.8 1 \
  && update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.9 2;

# Make sure we're using the latest pip
RUN pip install --upgrade pip \
  && pip install virtualenv

WORKDIR /app/

COPY ./package.json ./
RUN npm install .
ENV PATH=/app/node_modules/.bin:$PATH
