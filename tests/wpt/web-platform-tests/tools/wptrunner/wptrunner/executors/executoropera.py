from ..webdriver_server import OperaDriverServer
from .base import WdspecExecutor, WdspecProtocol


class OperaDriverProtocol(WdspecProtocol):
    server_cls = OperaDriverServer


class OperaDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = OperaDriverProtocol
