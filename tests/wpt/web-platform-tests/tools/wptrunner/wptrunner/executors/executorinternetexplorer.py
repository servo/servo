from ..webdriver_server import InternetExplorerDriverServer
from .base import WdspecExecutor, WdspecProtocol


class InternetExplorerDriverProtocol(WdspecProtocol):
    server_cls = InternetExplorerDriverServer


class InternetExplorerDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = InternetExplorerDriverProtocol
