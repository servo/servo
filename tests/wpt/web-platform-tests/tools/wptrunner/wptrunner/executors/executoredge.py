from ..webdriver_server import EdgeDriverServer
from .base import WdspecExecutor, WdspecProtocol


class EdgeDriverProtocol(WdspecProtocol):
    server_cls = EdgeDriverServer


class EdgeDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = EdgeDriverProtocol
