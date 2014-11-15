# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -e

servo_root=$(pwd)

PYTHON=$(which python2 2> /dev/null || echo python)
VIRTUALENV=$(which virtualenv2 2> /dev/null || echo virtualenv)

test -d _virtualenv || $VIRTUALENV _virtualenv -p $PYTHON
test -d $servo_root/tests/wpt/metadata || mkdir -p $servo_root/tests/wpt/metadata
test -d $servo_root/tests/wpt/prefs || mkdir -p $servo_root/tests/wpt/prefs
source _virtualenv/bin/activate
if [[ $* == *--update-manifest* ]]; then
    (python -c "import html5lib" &>/dev/null) || pip install html5lib
fi
(python -c "import wptrunner"  &>/dev/null) || pip install 'wptrunner==1.7'

python $servo_root/tests/wpt/run.py \
  --config $servo_root/tests/wpt/config.ini \
  --binary target/servo \
  --log-mach - \
  "$@"
