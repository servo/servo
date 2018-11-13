#!/usr/bin/env sh
set -ex

SCRIPT_DIR=$(dirname $(readlink -f "$0"))
WPT_ROOT=$(readlink -f $SCRIPT_DIR/..)
cd $WPT_ROOT

main() {
    cd $WPT_ROOT/css

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

    # Run the build script
    $VENV/bin/python tools/build.py "$@"
}

main "$@"
