# mypy: allow-untyped-defs

import os

from .executorwebdriver import WebDriverProtocol, WebDriverTestharnessExecutor, WebDriverRefTestExecutor

webdriver = None
ServoCommandExtensions = None

here = os.path.dirname(__file__)


def do_delayed_imports():
    global webdriver
    import webdriver

    global ServoCommandExtensions

    class ServoCommandExtensions:
        def __init__(self, session):
            self.session = session

        @webdriver.client.command
        def get_prefs(self, *prefs):
            body = {"prefs": list(prefs)}
            return self.session.send_session_command("POST", "servo/prefs/get", body)

        @webdriver.client.command
        def set_prefs(self, prefs):
            body = {"prefs": prefs}
            return self.session.send_session_command("POST", "servo/prefs/set", body)

        @webdriver.client.command
        def reset_prefs(self, *prefs):
            body = {"prefs": list(prefs)}
            return self.session.send_session_command("POST", "servo/prefs/reset", body)

        def change_prefs(self, old_prefs, new_prefs):
            # Servo interprets reset with an empty list as reset everything
            if old_prefs:
                self.reset_prefs(*old_prefs.keys())
            self.set_prefs({k: parse_pref_value(v) for k, v in new_prefs.items()})


# See parse_pref_from_command_line() in components/config/opts.rs
def parse_pref_value(value):
    if value == "true":
        return True
    if value == "false":
        return False
    try:
        return float(value)
    except ValueError:
        return value


class ServoWebDriverProtocol(WebDriverProtocol):
    def __init__(self, executor, browser, capabilities, **kwargs):
        do_delayed_imports()
        WebDriverProtocol.__init__(self, executor, browser, capabilities, **kwargs)

    def connect(self):
        """Connect to browser via WebDriver and crete a WebDriver session."""
        self.logger.debug("Connecting to WebDriver on URL: %s" % self.url)

        host, port = self.url.split(":")[1].strip("/"), self.url.split(':')[-1].strip("/")

        capabilities = {"alwaysMatch": self.capabilities}
        self.webdriver = webdriver.Session(host, port,
                                           capabilities=capabilities,
                                           enable_bidi=self.enable_bidi,
                                           extension=ServoCommandExtensions)
        self.webdriver.start()


class ServoWebDriverTestharnessExecutor(WebDriverTestharnessExecutor):
    supports_testdriver = True
    protocol_cls = ServoWebDriverProtocol

    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 close_after_done=True, capabilities={}, debug_info=None,
                 **kwargs):
        WebDriverTestharnessExecutor.__init__(self, logger, browser, server_config,
                                              timeout_multiplier, capabilities=capabilities,
                                              debug_info=debug_info, close_after_done=close_after_done,
                                              cleanup_after_test=False)

    def on_environment_change(self, new_environment):
        self.protocol.webdriver.extension.change_prefs(
            self.last_environment.get("prefs", {}),
            new_environment.get("prefs", {})
        )


class ServoWebDriverRefTestExecutor(WebDriverRefTestExecutor):
    protocol_cls = ServoWebDriverProtocol

    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, capabilities={}, debug_info=None,
                 **kwargs):
        WebDriverRefTestExecutor.__init__(self, logger, browser, server_config,
                                          timeout_multiplier, screenshot_cache,
                                          capabilities=capabilities,
                                          debug_info=debug_info)

    def on_environment_change(self, new_environment):
        self.protocol.webdriver.extension.change_prefs(
            self.last_environment.get("prefs", {}),
            new_environment.get("prefs", {})
        )
