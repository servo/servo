from ..webdriver_server import SafariDriverServer
from .base import WdspecExecutor, WdspecProtocol


class SafariDriverProtocol(WdspecProtocol):
    server_cls = SafariDriverServer


class SafariDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = SafariDriverProtocol
