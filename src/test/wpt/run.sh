# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

cd $2

test -d _virtualenv || virtualenv _virtualenv
test -d $1/src/test/wpt/metadata || mkdir -p $1/src/test/wpt/metadata
test -d $1/src/test/wpt/prefs || mkdir -p $1/src/test/wpt/prefs
source _virtualenv/bin/activate
(python -c "import html5lib" &>/dev/null) || pip install html5lib
(python -c "import wptrunner"  &>/dev/null) || pip install wptrunner
python $1/src/test/wpt/run.py --binary $2/../servo
