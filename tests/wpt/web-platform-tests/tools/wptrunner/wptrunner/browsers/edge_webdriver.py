from .base import inherit
from . import edge

from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor)  # noqa: F401


inherit(edge, globals(), "edge_webdriver")

# __wptrunner__ magically appears from inherit, F821 is undefined name
__wptrunner__["executor"]["testharness"] = "WebDriverTestharnessExecutor"  # noqa: F821
__wptrunner__["executor"]["reftest"] = "WebDriverRefTestExecutor"  # noqa: F821
