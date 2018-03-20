import os
import sys

from os.path import join, dirname

import mock
import pytest

from .base import all_products, active_products

sys.path.insert(0, join(dirname(__file__), "..", "..", "..", ".."))  # repo root

from tools import localpaths

import sslutils

from wptrunner import environment
from wptrunner import products

test_paths = {"/": {"tests_path": join(dirname(__file__), "..", "..", "..", "..")}}  # repo root
environment.do_delayed_imports(None, test_paths)


@active_products("product")
def test_load_active_product(product):
    """test we can successfully load the product of the current testenv"""
    products.load_product({}, product)
    # test passes if it doesn't throw


@all_products("product")
def test_load_all_products(product):
    """test every product either loads or throws ImportError"""
    try:
        products.load_product({}, product)
    except ImportError:
        pass


@active_products("product", marks={
    "sauce": pytest.mark.skip("needs env extras kwargs"),
})
def test_server_start_config(product):
    (check_args,
     target_browser_cls, get_browser_kwargs,
     executor_classes, get_executor_kwargs,
     env_options, get_env_extras, run_info_extras) = products.load_product({}, product)

    env_extras = get_env_extras()

    with mock.patch.object(environment.serve, "start") as start:
        with environment.TestEnvironment(test_paths,
                                         sslutils.environments["none"](None),
                                         False,
                                         None,
                                         env_options,
                                         env_extras) as test_environment:
            start.assert_called_once()
            args = start.call_args
            config = args[0][0]
            if "server_host" in env_options:
                assert config["server_host"] == env_options["server_host"]
            else:
                assert config["server_host"] == config["browser_host"]
            assert isinstance(config["bind_address"], bool)
