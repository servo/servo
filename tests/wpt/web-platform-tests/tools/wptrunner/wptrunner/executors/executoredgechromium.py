from ..webdriver_server import EdgeChromiumDriverServer
from .base import WdspecExecutor, WdspecProtocol


class EdgeChromiumDriverProtocol(WdspecProtocol):
    server_cls = EdgeChromiumDriverServer


class EdgeChromiumDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = EdgeChromiumDriverProtocol
