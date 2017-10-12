from ..webdriver_server import ChromeDriverServer
from .base import WdspecExecutor, WebDriverProtocol


class ChromeDriverProtocol(WebDriverProtocol):
    server_cls = ChromeDriverServer


class ChromeDriverWdspecExecutor(WdspecExecutor):
    protocol_cls = ChromeDriverProtocol
