# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

set -e

wpt_root=$(dirname $0)

PYTHON=$(which python2 2> /dev/null || echo python)
VIRTUALENV=$(which virtualenv2 2> /dev/null || echo virtualenv)

test -d $wpt_root/_virtualenv || $VIRTUALENV $wpt_root/_virtualenv -p $PYTHON
test -d $wpt_root/metadata || mkdir -p $wpt_root/metadata
test -d $wpt_root/prefs || mkdir -p $wpt_root/prefs
source $wpt_root/_virtualenv/bin/activate
if [[ $* == *--update-manifest* ]]; then
    (python -c "import html5lib" &>/dev/null) || pip install html5lib
fi
(python -c "import wptrunner"  &>/dev/null) || pip install 'wptrunner==1.8'

sed 's$manager_group.unexpected_count() == 0$unexpected_total == 0$' \
  $wpt_root/_virtualenv/lib/python2.7/site-packages/wptrunner/wptrunner.py \
  > $wpt_root/_virtualenv/lib/python2.7/site-packages/wptrunner/wptrunner.py.new
mv $wpt_root/_virtualenv/lib/python2.7/site-packages/wptrunner/wptrunner.py.new \
  $wpt_root/_virtualenv/lib/python2.7/site-packages/wptrunner/wptrunner.py

python $wpt_root/run.py \
  --config $wpt_root/config.ini \
  --binary $wpt_root/../../components/servo/target/servo \
  --log-mach - \
  "$@"
