# This file exists to allow `python wpt <command>` to work on Windows:
# https://github.com/web-platform-tests/wpt/pull/6907
exec(compile(open("wpt", "r").read(), "wpt", 'exec'))
