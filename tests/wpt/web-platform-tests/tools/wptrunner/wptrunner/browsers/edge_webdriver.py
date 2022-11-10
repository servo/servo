from .base import NullBrowser  # noqa: F401
from .edge import (EdgeBrowser,  # noqa: F401
                   check_args,  # noqa: F401
                   browser_kwargs,  # noqa: F401
                   executor_kwargs,  # noqa: F401
                   env_extras,  # noqa: F401
                   env_options,  # noqa: F401
                   run_info_extras,  # noqa: F401
                   get_timeout_multiplier)  # noqa: F401

from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor)  # noqa: F401


__wptrunner__ = {"product": "edge_webdriver",
                 "check_args": "check_args",
                 "browser": "EdgeBrowser",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor",
                              "wdspec": "WdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "run_info_extras": "run_info_extras",
                 "timeout_multiplier": "get_timeout_multiplier"}
