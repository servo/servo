import os
import sys

here = os.path.abspath(os.path.split(__file__)[0])
repo_root = os.path.abspath(os.path.join(here, os.pardir))

sys.path.insert(0, os.path.join(repo_root, "tools"))
sys.path.insert(0, os.path.join(repo_root, "tools", "six"))
sys.path.insert(0, os.path.join(repo_root, "tools", "html5lib"))
sys.path.insert(0, os.path.join(repo_root, "tools", "wptserve"))
sys.path.insert(0, os.path.join(repo_root, "tools", "pywebsocket", "src"))
