set -ex

SCRIPT_DIR=$(dirname $(readlink -f "$0"))
WPT_ROOT=$(readlink -f $SCRIPT_DIR/../..)
cd $WPT_ROOT

mkdir -p ~/meta
./wpt manifest -p ~/meta/MANIFEST.json
./wpt lint
