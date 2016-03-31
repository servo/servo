#!/bin/bash

# this assumes plugins are installed as sister directories

set -e
cd ../pytest-pep8
py.test
cd ../pytest-instafail
py.test 
cd ../pytest-cache
py.test
cd ../pytest-xprocess
py.test
#cd ../pytest-cov
#py.test
cd ../pytest-capturelog
py.test
cd ../pytest-xdist
py.test

