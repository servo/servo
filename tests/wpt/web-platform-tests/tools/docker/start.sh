#!/bin/bash

cd web-platform-tests
git pull --depth=50

sudo sh -c './wpt make-hosts-file >> /etc/hosts'

# Install Chome dev
deb_archive=google-chrome-unstable_current_amd64.deb
wget https://dl.google.com/linux/direct/$deb_archive

sudo gdebi -n $deb_archive

sudo Xvfb $DISPLAY -screen 0 ${SCREEN_WIDTH}x${SCREEN_HEIGHT}x${SCREEN_DEPTH} &
