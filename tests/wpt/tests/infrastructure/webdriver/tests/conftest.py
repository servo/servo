import os
import sys
# Hack to avoid duplicating the conftest file
wdpath = os.path.abspath(os.path.join(os.path.dirname(__file__),
                                      "../../../webdriver/"))
sys.path.insert(0, wdpath)
from tests.conftest import *
