import os
import sys

here = os.path.abspath(os.path.split(__file__)[0])
sys.path.insert(0, os.path.join(here, os.pardir, os.pardir, os.pardir))

import localpaths as _localpaths  # noqa: F401
