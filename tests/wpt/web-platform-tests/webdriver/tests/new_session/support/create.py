# Note that we can only test things here all implementations must support
valid_data = [
    ("acceptInsecureCerts", [
        False, None,
    ]),
    ("browserName", [
        None,
    ]),
    ("browserVersion", [
        None,
    ]),
    ("platformName", [
        None,
    ]),
    ("pageLoadStrategy", [
        None,
        "none",
        "eager",
        "normal",
    ]),
    ("proxy", [
        None,
    ]),
    ("timeouts", [
        None, {},
        {"script": 0, "pageLoad": 2.0, "implicit": 2**53 - 1},
        {"script": 50, "pageLoad": 25},
        {"script": 500},
    ]),
    ("unhandledPromptBehavior", [
        "dismiss",
        "accept",
        None,
    ]),
    ("test:extension", [
        None, False, "abc", 123, [],
        {"key": "value"},
    ]),
]

invalid_data = [
    ("acceptInsecureCerts", [
        1, [], {}, "false",
    ]),
    ("browserName", [
        1, [], {}, False,
    ]),
    ("browserVersion", [
        1, [], {}, False,
    ]),
    ("platformName", [
        1, [], {}, False,
    ]),
    ("pageLoadStrategy", [
        1, [], {}, False,
        "invalid",
        "NONE",
        "Eager",
        "eagerblah",
        "interactive",
        " eager",
        "eager "]),
    ("proxy", [
        1, [], "{}",
        {"proxyType": "SYSTEM"},
        {"proxyType": "systemSomething"},
        {"proxy type": "pac"},
        {"proxy-Type": "system"},
        {"proxy_type": "system"},
        {"proxytype": "system"},
        {"PROXYTYPE": "system"},
        {"proxyType": None},
        {"proxyType": 1},
        {"proxyType": []},
        {"proxyType": {"value": "system"}},
        {" proxyType": "system"},
        {"proxyType ": "system"},
        {"proxyType ": " system"},
        {"proxyType": "system "},
    ]),
    ("timeouts", [
        1, [], "{}", False,
        {"invalid": 10},
        {"PAGELOAD": 10},
        {"page load": 10},
        {" pageLoad": 10},
        {"pageLoad ": 10},
        {"pageLoad": None},
        {"pageLoad": False},
        {"pageLoad": []},
        {"pageLoad": "10"},
        {"pageLoad": 2.5},
        {"pageLoad": -1},
        {"pageLoad": 2**53},
        {"pageLoad": {"value": 10}},
        {"pageLoad": 10, "invalid": 10},
    ]),
    ("unhandledPromptBehavior", [
        1, [], {}, False,
        "DISMISS",
        "dismissABC",
        "Accept",
        " dismiss",
        "dismiss ",
    ])
]

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
