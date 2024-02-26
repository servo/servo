from urllib.parse import urlunsplit

import pytest


@pytest.fixture
def origin(server_config, domain_value):
    def origin(protocol="https", domain="", subdomain=""):
        return urlunsplit((protocol, domain_value(domain, subdomain), "", "", ""))

    return origin
