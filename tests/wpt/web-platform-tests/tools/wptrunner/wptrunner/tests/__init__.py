import os
import sys

here = os.path.abspath(os.path.dirname(__file__))
sys.path.insert(0, os.path.join(here, os.pardir, os.pardir, os.pardir))

import localpaths as _localpaths  # noqa: F401
