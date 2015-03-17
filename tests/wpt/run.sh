# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -e

wpt_root=$(dirname $0)
binary_dir=$wpt_root/../../components/servo/target
if [[ $1 ==  "--release" ]]; then
    binary_dir=$binary_dir/release
    shift
fi

PYTHON=$(which python2 2> /dev/null || echo python)
VIRTUALENV=$(which virtualenv2 2> /dev/null || echo virtualenv)

test -d $wpt_root/_virtualenv || $VIRTUALENV $wpt_root/_virtualenv -p $PYTHON
test -d $wpt_root/metadata || mkdir -p $wpt_root/metadata
test -d $wpt_root/prefs || mkdir -p $wpt_root/prefs
source $wpt_root/_virtualenv/bin/activate
if [[ $* == *--update-manifest* ]]; then
    (python -c "import html5lib" &>/dev/null) || pip install html5lib
fi
(python -c "import wptrunner"  &>/dev/null) || pip install 'wptrunner==1.13'

python $wpt_root/run.py \
  --config $wpt_root/config.ini \
  --binary $binary_dir/servo \
  --log-mach - \
  "$@"
