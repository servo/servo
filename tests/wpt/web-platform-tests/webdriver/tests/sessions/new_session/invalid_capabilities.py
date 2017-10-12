#META: timeout=long

import pytest
from webdriver import error

from conftest import product, flatten


@pytest.mark.parametrize("value", [None, 1, "{}", []])
def test_invalid_capabilites(new_session, value):
    with pytest.raises(error.InvalidArgumentException):
        new_session({"capabilities": value})


@pytest.mark.parametrize("value", [None, 1, "{}", []])
def test_invalid_always_match(new_session, value):
    with pytest.raises(error.InvalidArgumentException):
        new_session({"capabilities": {"alwaysMatch": value}})


@pytest.mark.parametrize("value", [None, 1, "[]", {}])
def test_invalid_first_match(new_session, value):
    with pytest.raises(error.InvalidArgumentException):
        new_session({"capabilities": {"firstMatch": value}})


invalid_data = [
    ("acceptInsecureCerts", [1, [], {}, "false"]),
    ("browserName", [1, [], {}, False]),
    ("browserVersion", [1, [], {}, False]),
    ("platformName", [1, [], {}, False]),
    ("pageLoadStrategy", [1, [], {}, False, "invalid", "NONE", "Eager", "eagerblah", "interactive",
                          " eager", "eager "]),
    ("proxy", [1, [], "{}", {"proxyType": "SYSTEM"}, {"proxyType": "systemSomething"},
               {"proxy type": "pac"}, {"proxy-Type": "system"}, {"proxy_type": "system"},
               {"proxytype": "system"}, {"PROXYTYPE": "system"}, {"proxyType": None},
               {"proxyType": 1}, {"proxyType": []}, {"proxyType": {"value": "system"}},
               {" proxyType": "system"}, {"proxyType ": "system"}, {"proxyType ": " system"},
               {"proxyType": "system "}]),
    ("timeouts", [1, [], "{}", {}, False, {"pageLOAD": 10}, {"page load": 10},
                  {"page load": 10}, {"pageLoad": "10"}, {"pageLoad": {"value": 10}},
                  {"invalid": 10}, {"pageLoad": -1}, {"pageLoad": 2**64},
                  {"pageLoad": None}, {"pageLoad": 1.1}, {"pageLoad": 10, "invalid": 10},
                  {" pageLoad": 10}, {"pageLoad ": 10}]),
    ("unhandledPromptBehavior", [1, [], {}, False, "DISMISS", "dismissABC", "Accept",
                                 " dismiss", "dismiss "])
]

@pytest.mark.parametrize("body", [lambda key, value: {"alwaysMatch": {key: value}},
                                  lambda key, value: {"firstMatch": [{key: value}]}])
@pytest.mark.parametrize("key,value", flatten(product(*item) for item in invalid_data))
def test_invalid(new_session, body, key, value):
    with pytest.raises(error.InvalidArgumentException):
        resp = new_session({"capabilities": body(key, value)})


invalid_extensions = [
    "firefox",
    "firefox_binary",
    "firefoxOptions",
    "chromeOptions",
    "automaticInspection",
    "automaticProfiling",
    "platform",
    "version",
    "browser",
    "platformVersion",
    "javascriptEnabled",
    "nativeEvents",
    "seleniumProtocol",
    "profile",
    "trustAllSSLCertificates",
    "initialBrowserUrl",
    "requireWindowFocus",
    "logFile",
    "logLevel",
    "safari.options",
    "ensureCleanSession",
]


@pytest.mark.parametrize("body", [lambda key, value: {"alwaysMatch": {key: value}},
                                  lambda key, value: {"firstMatch": [{key: value}]}])
@pytest.mark.parametrize("key", invalid_extensions)
def test_invalid(new_session, body, key):
    with pytest.raises(error.InvalidArgumentException):
        resp = new_session({"capabilities": body(key, {})})

