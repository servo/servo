# mypy: allow-untyped-defs, allow-untyped-calls

import sys
import warnings
from os.path import dirname, join
from unittest import mock
from unittest.mock import Mock

import pytest

from .. import environment, products, wptcommandline
from .base import active_products, all_products

wpt_root = join(dirname(__file__), "..", "..", "..", "..")

test_paths = {"/": wptcommandline.TestRoot(wpt_root, wpt_root)}
environment.do_delayed_imports(None, test_paths)


@active_products("product")
def test_load_active_product(product):
    """test we can successfully load the product of the current testenv"""
    products.Product.from_product_name(product)
    # test passes if it doesn't throw


@all_products("product")
def test_load_all_products(product):
    """test every product either loads or throws ImportError"""
    with warnings.catch_warnings():
        # This acts to ensure that we don't get a DeprecationWarning here.
        warnings.filterwarnings(
            "error",
            message=r"Use Product\.from_product_name",
            category=DeprecationWarning,
        )
        try:
            products.Product.from_product_name(product)
        except ImportError:
            pass


def test_product_from_name_unknown():
    """Test that Product.from_product_name raises ValueError for unknown products."""
    with pytest.raises(ValueError, match="Unknown product"):
        products.Product.from_product_name("nonexistent_product")


@active_products("product", marks={
    "sauce": pytest.mark.skip("needs env extras kwargs"),
})
def test_server_start_config(product):
    product_data = products.Product.from_product_name(product)

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


@pytest.mark.parametrize("enabled", [None, False, True])
def test_websocket_h3_environment(enabled):
    parser = wptcommandline.create_parser()
    assert parser.parse_args([]).enable_websocket_h3 is None
    assert parser.parse_args(["--enable-websocket-h3"]).enable_websocket_h3 is True
    assert parser.parse_args(["--no-enable-websocket-h3"]).enable_websocket_h3 is False

    with mock.patch.object(environment.serve, "start") as start:
        with environment.TestEnvironment(
                test_paths, 1, False, False, None,
                {"enable_websocket_h3": True},
                environment.serve.ConfigBuilder(mock.Mock()).ssl, [],
                enable_websocket_h3=enabled) as env:
            assert env.config.ports["websocket-h3"] == [11001]
            assert env.config.ports["websocket-h3-https"] == [11001]
            assert start.call_args.kwargs["websocket_h3"] is (enabled is True)

    with mock.patch.object(environment.serve, "start"):
        with environment.TestEnvironment(
                test_paths, 1, False, False, None,
                {}, {"type": "none"}, [],
                enable_websocket_h3=True) as env:
            assert not env.config.ports["websocket-h3"]
            assert not env.config.ports["websocket-h3-https"]


def test_websocket_h3_readiness():
    env = object.__new__(environment.TestEnvironment)
    server = mock.Mock()
    server.is_alive.return_value = True
    env.servers = {"websocket-h3": [(11001, server)]}
    env.config = {"server_host": "127.0.0.1"}
    env.test_server_port = True

    with mock.patch.object(environment, "websocket_h3_server_is_running",
                           return_value=False) as probe, \
         mock.patch.object(environment.socket, "socket") as tcp_socket:
        assert env.test_servers() == ([], [("websocket-h3", 11001)])
        probe.assert_called_once_with("127.0.0.1", 11001, timeout=5)
        tcp_socket.assert_not_called()

    with mock.patch.object(environment, "websocket_h3_server_is_running",
                           return_value=True):
        assert env.test_servers() == ([], [])


def test_default_if_none_descriptor_with_none() -> None:
    """Test that _DefaultIfNone descriptor converts None to default"""
    product = products.Product(
        name="test",
        browser_classes={None: mock.MagicMock()},
        check_args=mock.MagicMock(),
        get_browser_kwargs=mock.MagicMock(),
        get_executor_kwargs=mock.MagicMock(),
        env_options={},
        get_env_extras=mock.MagicMock(),
        get_timeout_multiplier=mock.MagicMock(),
        executor_classes={},
        run_info_extras=None,  # This would fail without the descriptor
        update_properties=None,  # This would fail without the descriptor
    )

    assert product.run_info_extras is products.default_run_info_extras
    assert product.update_properties == (["product"], {})
    assert product.add_arguments is products.default_add_arguments


def test_default_if_none_descriptor_with_provided_values() -> None:
    """Test that _DefaultIfNone descriptor preserves provided values"""
    def custom_run_info_extras(logger, **kwargs):
        return {"custom": "value"}

    def custom_add_arguments(parser):
        group = parser.add_argument_group("Test-specific")
        group.add_argument("--test-option", help="A test option")
        group.add_argument("--another-option", type=int, default=42)

    custom_update_properties = (["custom"], {"prop": ["value"]})

    product = products.Product(
        name="test",
        browser_classes={None: mock.MagicMock()},
        check_args=mock.MagicMock(),
        get_browser_kwargs=mock.MagicMock(),
        get_executor_kwargs=mock.MagicMock(),
        env_options={},
        get_env_extras=mock.MagicMock(),
        get_timeout_multiplier=mock.MagicMock(),
        executor_classes={},
        run_info_extras=custom_run_info_extras,
        update_properties=custom_update_properties,
        add_arguments=custom_add_arguments,
    )

    assert product.run_info_extras is custom_run_info_extras
    assert product.update_properties is custom_update_properties
    assert product.add_arguments is custom_add_arguments


def test_create_parser_calls_add_arguments() -> None:
    """Test that create_parser calls add_arguments for available products."""
    def custom_add_arguments(parser):
        parser.add_argument("--custom-product-option")

    mock_product = mock.MagicMock(spec=products.Product)
    mock_product.add_arguments = custom_add_arguments

    with mock.patch("wptrunner.products.get_all_products", return_value={"mockbrowser": mock.MagicMock()}):
        with mock.patch("wptrunner.products.Product.from_product_name", return_value=mock_product):
            parser = wptcommandline.create_parser(product_choices=["mockbrowser"])

    args = parser.parse_args(["--product", "mockbrowser", "--custom-product-option", "value"])
    assert args.custom_product_option == "value"


@pytest.mark.parametrize("product", products.BUILTIN_PRODUCTS)
def test_get_product_names_includes_builtins(product):
    """Test that built-in products are included."""
    names = products.get_all_products()
    assert product in names


def test_entry_point_is_callable():
    """Test that loading a non-callable entry point raises TypeError."""
    mock_ep = Mock()
    mock_ep.name = "badproduct"
    mock_ep.load.return_value = "not callable"

    if sys.version_info >= (3, 10):
        return_value = [mock_ep]
    else:
        return_value = {"wptrunner.products": [mock_ep]}

    with mock.patch("wptrunner.products.entry_points", autospec=True, return_value=return_value):
        with pytest.raises(TypeError):
            products.Product.from_product_name("badproduct")


def test_entry_point_returns_product():
    """Test that entry point callable must return Product instance."""
    mock_ep = Mock()
    mock_ep.name = "badproduct"
    mock_ep.load.return_value = lambda: "not a Product instance"

    if sys.version_info >= (3, 10):
        return_value = [mock_ep]
    else:
        return_value = {"wptrunner.products": [mock_ep]}

    with mock.patch("wptrunner.products.entry_points", autospec=True, return_value=return_value):
        with pytest.raises(TypeError, match="instead of Product"):
            products.Product.from_product_name("badproduct")


def test_entry_point_product_name_validation():
    """Test that entry point name must match Product.name."""
    def get_product_wrong_name():
        mock_product = Mock(spec=products.Product)
        mock_product.name = "actual_name"
        return mock_product

    mock_ep = Mock()
    mock_ep.name = "wrong_name"
    mock_ep.load.return_value = get_product_wrong_name

    if sys.version_info >= (3, 10):
        return_value = [mock_ep]
    else:
        return_value = {"wptrunner.products": [mock_ep]}

    with mock.patch("wptrunner.products.entry_points", autospec=True, return_value=return_value):
        with pytest.raises(ValueError, match="name="):
            products.Product.from_product_name("wrong_name")


@active_products("product")
def test_entry_point_shadows_builtin(product):
    """Test that external entry points can shadow built-in products."""
    if product not in products.BUILTIN_PRODUCTS:
        pytest.skip("Only applies to built-ins")

    ep = next(ep for ep in products._BUILTIN_ENTRY_POINTS if ep.name == product)

    mock_ep = Mock(wraps=ep)
    mock_ep.name = ep.name
    mock_ep.load.return_value = lambda: None

    if sys.version_info >= (3, 10):
        return_value = [mock_ep]
    else:
        return_value = {"wptrunner.products": [mock_ep]}

    with mock.patch("wptrunner.products.entry_points", autospec=True, return_value=return_value):
        with pytest.raises(TypeError, match="instead of Product"):
            product = products.Product.from_product_name(product)
        mock_ep.load.assert_called_once()
