# mypy: allow-untyped-defs, allow-untyped-calls

import logging
from os.path import join, dirname

import pytest

from wptserve.config import ConfigBuilder
from ..base import active_products
from wptrunner import environment, products, testloader, wptcommandline

wpt_root = join(dirname(__file__), "..", "..", "..", "..")

test_paths = {"/": wptcommandline.TestRoot(wpt_root, wpt_root)}
environment.do_delayed_imports(None, test_paths)

logger = logging.getLogger()


@active_products("product")
def test_webkitgtk_certificate_domain_list(product):

    def domain_is_inside_certificate_list_cert(domain_to_find, webkitgtk_certificate_list, cert_file):
        for domain in webkitgtk_certificate_list:
            if domain["host"] == domain_to_find and domain["certificateFile"] == cert_file:
                return True
        return False

    if product not in ["epiphany", "webkit", "webkitgtk_minibrowser"]:
        pytest.skip("%s doesn't support certificate_domain_list" % product)

    product_data = products.Product({}, product)

    cert_file = "/home/user/wpt/tools/certs/cacert.pem"
    valid_domains_test = ["a.example.org", "b.example.org", "example.org",
                          "a.example.net", "b.example.net", "example.net"]
    invalid_domains_test = ["x.example.org", "y.example.org", "example.it",
                            "x.example.net", "y.example.net", "z.example.net"]
    kwargs = {}
    kwargs["timeout_multiplier"] = 1
    kwargs["debug_info"] = None
    kwargs["host_cert_path"] = cert_file
    kwargs["webkit_port"] = "gtk"
    kwargs["binary"] = None
    kwargs["webdriver_binary"] = None
    kwargs["pause_after_test"] = False
    kwargs["pause_on_unexpected"] = False
    kwargs["debug_test"] = False
    kwargs["subsuite"] = testloader.Subsuite("", config={})
    with ConfigBuilder(logger,
                       browser_host="example.net",
                       alternate_hosts={"alt": "example.org"},
                       subdomains={"a", "b"},
                       not_subdomains={"x", "y"}) as env_config:

        # We don't want to actually create a test environment; the get_executor_kwargs
        # function only really wants an object with the config key

        class MockEnvironment:
            def __init__(self, config):
                self.config = config

        executor_args = product_data.get_executor_kwargs(None,
                                                         None,
                                                         MockEnvironment(env_config),
                                                         {},
                                                         **kwargs)
        assert 'capabilities' in executor_args
        assert 'webkitgtk:browserOptions' in executor_args['capabilities']
        assert 'certificates' in executor_args['capabilities']['webkitgtk:browserOptions']
        cert_list = executor_args['capabilities']['webkitgtk:browserOptions']['certificates']
        for valid_domain in valid_domains_test:
            assert domain_is_inside_certificate_list_cert(valid_domain, cert_list, cert_file)
            assert not domain_is_inside_certificate_list_cert(valid_domain, cert_list, cert_file + ".backup_non_existent")
        for invalid_domain in invalid_domains_test:
            assert not domain_is_inside_certificate_list_cert(invalid_domain, cert_list, cert_file)
            assert not domain_is_inside_certificate_list_cert(invalid_domain, cert_list, cert_file + ".backup_non_existent")
