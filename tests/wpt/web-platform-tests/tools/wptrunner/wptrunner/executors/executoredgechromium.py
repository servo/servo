from ..webdriver_server import EdgeChromiumDriverServer
from .base import WdspecExecutor, WebDriverProtocol


class EdgeChromiumDriverProtocol(WebDriverProtocol):
    server_cls = EdgeChromiumDriverServer


class EdgeChromiumDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = EdgeChromiumDriverProtocol
