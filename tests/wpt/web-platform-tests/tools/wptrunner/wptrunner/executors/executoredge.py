from ..webdriver_server import EdgeDriverServer
from .base import WdspecExecutor, WebDriverProtocol


class EdgeDriverProtocol(WebDriverProtocol):
    server_cls = EdgeDriverServer


class EdgeDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = EdgeDriverProtocol
