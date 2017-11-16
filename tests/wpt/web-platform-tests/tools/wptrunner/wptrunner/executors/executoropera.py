from ..webdriver_server import OperaDriverServer
from .base import WdspecExecutor, WebDriverProtocol


class OperaDriverProtocol(WebDriverProtocol):
    server_cls = OperaDriverServer


class OperaDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = OperaDriverProtocol
