FROM python:3.11

# No interactive frontend during docker build
ENV DEBIAN_FRONTEND=noninteractive \
    DEBCONF_NONINTERACTIVE_SEEN=true

RUN apt-get -qqy update \
  && apt-get -qqy install git npm

WORKDIR /app/

COPY package.json requirements.txt ./

RUN npm install .
ENV PATH=/app/node_modules/.bin:$PATH

# Use venv to create a virtual environment with the docs dependencies installed,
# setting the environment variables needed for this to always be active. The
# `./wpt build-docs` then uses this venv with --skip-venv-setup.
ENV VIRTUAL_ENV=/app/venv
RUN python3 -m venv $VIRTUAL_ENV
ENV PATH=$VIRTUAL_ENV/bin:$PATH
RUN pip install -r requirements.txt
