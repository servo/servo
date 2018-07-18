from ..webdriver_server import SafariDriverServer
from .base import WdspecExecutor, WebDriverProtocol


class SafariDriverProtocol(WebDriverProtocol):
    server_cls = SafariDriverServer


class SafariDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = SafariDriverProtocol
