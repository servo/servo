#!/bin/bash -e

if [[ $USE_OPTIONAL != "true" && $USE_OPTIONAL != "false" ]]; then
  echo "fatal: \$USE_OPTIONAL not set to true or false. Exiting."
  exit 1
fi

pip install -r requirements-test.txt

if [[ $USE_OPTIONAL == "true" && $TRAVIS_PYTHON_VERSION != "pypy" ]]; then
  if [[ $TRAVIS_PYTHON_VERSION == "2.6" ]]; then
    pip install --allow-external Genshi --allow-insecure Genshi -r requirements-optional-2.6.txt
  else
    pip install --allow-external Genshi --allow-insecure Genshi -r requirements-optional-cpython.txt
  fi
fi
