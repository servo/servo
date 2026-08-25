import os
import sys

base = os.path.dirname(__file__)
webdriver_path = os.path.abspath(os.path.join(base, "..", "..", "..", "webdriver"))
sys.path.insert(0, os.path.join(webdriver_path))

from tests.conftest import *
