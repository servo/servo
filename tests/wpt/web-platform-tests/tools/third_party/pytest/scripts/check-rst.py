
from __future__ import print_function

import subprocess
import glob
import sys

sys.exit(subprocess.call([
    'rst-lint', '--encoding', 'utf-8',
    'CHANGELOG.rst', 'HOWTORELEASE.rst', 'README.rst',
] + glob.glob('changelog/[0-9]*.*')))
