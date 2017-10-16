# Note that we can only test things here all implementations must support
valid_data = [
    ("acceptInsecureCerts", [False, None]),
    ("browserName", [None]),
    ("browserVersion", [None]),
    ("platformName", [None]),
    ("pageLoadStrategy", ["none", "eager", "normal", None]),
    ("proxy", [None]),
    ("unhandledPromptBehavior", ["dismiss", "accept", None]),
    ("test:extension", [True, "abc", 123, [], {"key": "value"}, None]),
]
