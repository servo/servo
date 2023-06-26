# mypy: allow-untyped-defs, allow-untyped-calls

from os.path import join, dirname
from unittest import mock

import pytest

from .base import all_products, active_products
from .. import environment
from .. import products

test_paths = {"/": {"tests_path": join(dirname(__file__), "..", "..", "..", "..")}}  # repo root
environment.do_delayed_imports(None, test_paths)


@active_products("product")
def test_load_active_product(product):
    """test we can successfully load the product of the current testenv"""
    products.Product({}, product)
    # test passes if it doesn't throw


@all_products("product")
def test_load_all_products(product):
    """test every product either loads or throws ImportError"""
    try:
        products.Product({}, product)
    except ImportError:
        pass


@active_products("product", marks={
    "sauce": pytest.mark.skip("needs env extras kwargs"),
})
def test_server_start_config(product):
    product_data = products.Product({}, product)

    env_extras = product_data.get_env_extras()

    with mock.patch.object(environment.serve, "start") as start:
        with environment.TestEnvironment(test_paths,
                                         1,
                                         False,
                                         False,
                                         None,
                                         product_data.env_options,
                                         {"type": "none"},
                                         env_extras):
            start.assert_called_once()
            args = start.call_args
            config = args[0][1]
            if "server_host" in product_data.env_options:
                assert config["server_host"] == product_data.env_options["server_host"]
            else:
                assert config["server_host"] == config["browser_host"]
            assert isinstance(config["bind_address"], bool)
