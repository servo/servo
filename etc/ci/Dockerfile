FROM ubuntu:vivid

# Required by mozjs to build
ENV SHELL /bin/sh

# Enable 'universe' since it is not enabled by default
RUN echo "deb http://archive.ubuntu.com/ubuntu vivid main universe" > /etc/apt/sources.list
RUN echo "deb http://archive.ubuntu.com/ubuntu vivid-updates main universe" >> /etc/apt/sources.list

# Install dependencies
RUN apt-get -y update
RUN apt-get install -y \
    cmake \
    curl \
    freeglut3-dev \
    g++ \
    git \
    gperf \
    libbz2-dev \
    libfreetype6-dev \
    libgl1-mesa-dri \
    libglib2.0-dev \
    libglu1-mesa-dev \
    libosmesa6-dev \
    libssl-dev \
    libxmu-dev \
    libxmu6 \
    python-virtualenv \
    xorg-dev

# Servo will be built in /build
RUN mkdir /build
WORKDIR /build
