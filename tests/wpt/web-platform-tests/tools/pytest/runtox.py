#!/usr/bin/env python

if __name__ == "__main__":
    import subprocess
    import sys
    subprocess.call([sys.executable, "-m", "tox",
                     "-i", "ALL=https://devpi.net/hpk/dev/",
                     "--develop"] + sys.argv[1:])
