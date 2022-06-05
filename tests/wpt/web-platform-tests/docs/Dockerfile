FROM ubuntu:20.04

# No interactive frontend during docker build
ENV DEBIAN_FRONTEND=noninteractive \
    DEBCONF_NONINTERACTIVE_SEEN=true

# Documentation is generated using Python 3.9 due to spinx-js not
# supporting 3.10: https://github.com/mozilla/sphinx-js/issues/186
RUN apt-get -qqy update \
  && apt-get -qqy install git npm python3.9 python3.9-venv

WORKDIR /app/

COPY package.json requirements.txt ./

RUN npm install .
ENV PATH=/app/node_modules/.bin:$PATH

# Use venv to create a virtual environment with the docs dependencies installed,
# setting the environment variables needed for this to always be active. The
# `./wpt build-docs` then uses this venv with --skip-venv-setup.
ENV VIRTUAL_ENV=/app/venv
RUN python3.9 -m venv $VIRTUAL_ENV
ENV PATH=$VIRTUAL_ENV/bin:$PATH
RUN pip install -r requirements.txt
