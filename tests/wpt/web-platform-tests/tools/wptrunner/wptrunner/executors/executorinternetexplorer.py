from ..webdriver_server import InternetExplorerDriverServer
from .base import WdspecExecutor, WebDriverProtocol


class InternetExplorerDriverProtocol(WebDriverProtocol):
    server_cls = InternetExplorerDriverServer


class InternetExplorerDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = InternetExplorerDriverProtocol
