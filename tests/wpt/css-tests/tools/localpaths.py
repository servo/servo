import os
import sys

here = os.path.abspath(os.path.split(__file__)[0])
repo_root = os.path.abspath(os.path.join(here, os.pardir))

sys.path.insert(0, os.path.join(here))
sys.path.insert(0, os.path.join(here, "six"))
sys.path.insert(0, os.path.join(here, "html5lib"))
sys.path.insert(0, os.path.join(here, "wptserve"))
sys.path.insert(0, os.path.join(here, "pywebsocket", "src"))
sys.path.insert(0, os.path.join(here, "py"))
sys.path.insert(0, os.path.join(here, "pytest"))
sys.path.insert(0, os.path.join(here, "webdriver"))
