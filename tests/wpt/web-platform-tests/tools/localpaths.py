import os
import sys

here = os.path.abspath(os.path.dirname(__file__))

sys.path.insert(0, os.path.join(here))
sys.path.insert(0, os.path.join(here, "wptserve"))
sys.path.insert(0, os.path.join(here, "third_party", "pywebsocket3"))
sys.path.insert(0, os.path.join(here, "third_party", "atomicwrites"))
sys.path.insert(0, os.path.join(here, "third_party", "attrs", "src"))
sys.path.insert(0, os.path.join(here, "third_party", "funcsigs"))
sys.path.insert(0, os.path.join(here, "third_party", "html5lib"))
sys.path.insert(0, os.path.join(here, "third_party", "zipp"))
sys.path.insert(0, os.path.join(here, "third_party", "more-itertools"))
sys.path.insert(0, os.path.join(here, "third_party", "packaging"))
sys.path.insert(0, os.path.join(here, "third_party", "pathlib2"))
sys.path.insert(0, os.path.join(here, "third_party", "pluggy", "src"))
sys.path.insert(0, os.path.join(here, "third_party", "py"))
sys.path.insert(0, os.path.join(here, "third_party", "pytest"))
sys.path.insert(0, os.path.join(here, "third_party", "pytest", "src"))
sys.path.insert(0, os.path.join(here, "third_party", "pytest-asyncio"))
sys.path.insert(0, os.path.join(here, "third_party", "six"))
sys.path.insert(0, os.path.join(here, "third_party", "webencodings"))
sys.path.insert(0, os.path.join(here, "third_party", "h2"))
sys.path.insert(0, os.path.join(here, "third_party", "hpack"))
sys.path.insert(0, os.path.join(here, "third_party", "hyperframe"))
sys.path.insert(0, os.path.join(here, "third_party", "certifi"))
sys.path.insert(0, os.path.join(here, "third_party", "hyper"))
sys.path.insert(0, os.path.join(here, "third_party", "websockets", "src"))
sys.path.insert(0, os.path.join(here, "third_party", "iniconfig", "src"))
if sys.version_info < (3, 8):
    sys.path.insert(0, os.path.join(here, "third_party", "importlib_metadata"))
sys.path.insert(0, os.path.join(here, "webdriver"))
sys.path.insert(0, os.path.join(here, "wptrunner"))

if sys.version_info[0] == 2:
    sys.path.insert(0, os.path.join(here, "third_party", "enum"))

# We can't import six until we've set the path above.
from six import ensure_text
repo_root = ensure_text(os.path.abspath(os.path.join(here, os.pardir)))
