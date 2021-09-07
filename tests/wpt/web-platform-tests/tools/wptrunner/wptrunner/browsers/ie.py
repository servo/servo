from .base import require_arg
from .base import NullBrowser, get_timeout_multiplier  # noqa: F401
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorinternetexplorer import InternetExplorerDriverWdspecExecutor  # noqa: F401

__wptrunner__ = {"product": "ie",
                 "check_args": "check_args",
                 "browser": "NullBrowser",
                 "executor": {"wdspec": "InternetExplorerDriverWdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "timeout_multiplier": "get_timeout_multiplier"}


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")

def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {"webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args")}

def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    options = {}
    options["requireWindowFocus"] = True
    capabilities = {}
    capabilities["se:ieOptions"] = options
    executor_kwargs = base_executor_kwargs(test_type, test_environment, run_info_data, **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["capabilities"] = capabilities
    return executor_kwargs

def env_extras(**kwargs):
    return []

def env_options():
    return {"supports_debugger": False}
