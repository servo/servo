from ..webdriver_server import WebKitDriverServer
from .base import WdspecExecutor, WdspecProtocol


class WebKitDriverProtocol(WdspecProtocol):
    server_cls = WebKitDriverServer


class WebKitDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = WebKitDriverProtocol
