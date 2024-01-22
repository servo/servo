import pytest

from urllib.parse import urlunsplit


@pytest.fixture
def origin(server_config, domain_value):
    def origin(protocol="https", domain="", subdomain=""):
        return urlunsplit((protocol, domain_value(domain, subdomain), "", "", ""))

    return origin


@pytest.fixture
def domain_value(server_config):
    def domain_value(domain="", subdomain=""):
        return server_config["domains"][domain][subdomain]

    return domain_value
