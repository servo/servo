FROM ubuntu:20.04

# No interactive frontend during docker build
ENV DEBIAN_FRONTEND=noninteractive \
    DEBCONF_NONINTERACTIVE_SEEN=true

# General requirements not in the base image
RUN apt-get -qqy update \
  && apt-get -qqy install \
    bridge-utils \
    bzip2 \
    ca-certificates \
    curl \
    dbus-x11 \
    earlyoom \
    fluxbox \
    gdebi \
    git \
    glib-networking-services \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-gl \
    libosmesa6-dev \
    libproxy1-plugin-webkit \
    libvirt-daemon-system \
    libvirt-clients \
    libunwind8 \
    libxcb-shape0-dev \
    locales \
    openjdk-8-jre-headless \
    pulseaudio \
    python3 \
    python3-dev \
    python3-pip \
    software-properties-common \
    qemu-kvm \
    tzdata \
    sudo \
    unzip \
    wget \
    xvfb

# python3.6 is not available by default in new versions of Ubuntu.
RUN apt-add-repository -y ppa:deadsnakes/ppa

# Ensure a `python` binary exists
RUN apt-get -qqy update \
    && apt-get install -qqy python-is-python3

# Installing just the deps of firefox and chrome is moderately tricky, so
# just install the default versions of them, and some extra deps we happen
# to know that chrome requires

RUN apt-get -qqy install \
    firefox \
    libnss3-tools \
    fonts-liberation \
    indicator-application \
    libappindicator1 \
    libappindicator3-1 \
    libdbusmenu-gtk3-4 \
    libindicator3-7 \
    libindicator7

RUN apt-get -y autoremove

RUN pip install --upgrade pip
RUN pip install virtualenv

ENV TZ "UTC"
RUN echo "${TZ}" > /etc/timezone \
  && dpkg-reconfigure --frontend noninteractive tzdata

# Set the locale
RUN locale-gen en_US.UTF-8
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8

RUN useradd test \
         --shell /bin/bash  \
         --create-home \
  && usermod -a -G sudo test \
  && usermod -a -G libvirt test \
  && usermod -a -G libvirt-qemu test \
  && echo 'ALL ALL = (ALL) NOPASSWD: ALL' >> /etc/sudoers \
  && echo 'test:secret' | chpasswd

ENV SCREEN_WIDTH 1280
ENV SCREEN_HEIGHT 1024
ENV SCREEN_DEPTH 24
ENV DISPLAY :99.0

USER test

WORKDIR /home/test

# Remove information on how to use sudo on login
RUN sudo echo ""

RUN mkdir -p /home/test/artifacts
RUN mkdir -p /home/test/bin

ENV PATH="/home/test/bin:/home/test/.local/bin:${PATH}"

WORKDIR /home/test/

COPY .bashrc /home/test/.bashrc

COPY start.sh /home/test/start.sh
COPY retry.py /home/test/bin/retry
