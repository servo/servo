set -ex

./manifest
./lint
./diff-manifest.py
