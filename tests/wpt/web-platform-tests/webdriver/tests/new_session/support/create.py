# Note that we can only test things here all implementations must support
valid_data = [
    ("acceptInsecureCerts", [False, None]),
    ("browserName", [None]),
    ("browserVersion", [None]),
    ("platformName", [None]),
    ("pageLoadStrategy", ["none", "eager", "normal", None]),
    ("proxy", [None]),
    ("timeouts", [{"script": 0, "pageLoad": 2.0, "implicit": 2**53 - 1},
                  {"script": 50, "pageLoad": 25},
                  {"script": 500},
                  {}]),
    ("unhandledPromptBehavior", ["dismiss", "accept", None]),
    ("test:extension", [True, "abc", 123, [], {"key": "value"}, None]),
]
