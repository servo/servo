from ..webdriver_server import WebKitDriverServer
from .base import WdspecExecutor, WebDriverProtocol


class WebKitDriverProtocol(WebDriverProtocol):
    server_cls = WebKitDriverServer


class WebKitDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = WebKitDriverProtocol
