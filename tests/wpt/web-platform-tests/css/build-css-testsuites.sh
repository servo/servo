#!/usr/bin/env sh
set -ex

cd "`dirname $0`"

if [ -z $VENV ]; then
    VENV=tools/_virtualenv
fi

# Create the virtualenv
if [ ! -d $VENV ]; then
    if [ -z $PYTHON ]; then
        command -v python
        if [ $? -eq 0 ]; then
            if [ `python -c 'import sys; print(sys.version[0:3])'` == "2.7" ]; then
                PYTHON=python
            fi
        fi
    fi

    if [ -z $PYTHON ]; then
        command -v python2
        if [ $? -eq 0 ]; then
            PYTHON=python2
        fi
    fi

    if [ -z $PYTHON ]; then
        echo "Please ensure Python 2.7 is installed"
        exit 1
    fi

    virtualenv -p $PYTHON $VENV || { echo "Please ensure virtualenv is installed"; exit 2; }
fi

# Install dependencies
$VENV/bin/pip install -r requirements.txt

# Fetch hg submodules if they're not there
if [ ! -d tools/apiclient ]; then
    $VENV/bin/hg clone https://hg.csswg.org/dev/apiclient tools/apiclient
fi

if [ ! -d tools/w3ctestlib ]; then
    $VENV/bin/hg clone https://hg.csswg.org/dev/w3ctestlib tools/w3ctestlib
fi

# Run the build script
$VENV/bin/python tools/build.py "$@"
