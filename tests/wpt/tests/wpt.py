# This file exists to allow `python wpt <command>` to work on Windows:
# https://github.com/web-platform-tests/wpt/pull/6907 and
# https://github.com/web-platform-tests/wpt/issues/23095
import os
abspath = os.path.abspath(__file__)
os.chdir(os.path.dirname(abspath))
exec(compile(open("wpt", "r").read(), "wpt", 'exec'))
