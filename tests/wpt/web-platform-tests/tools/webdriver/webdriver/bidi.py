import copy
import websockets

from . import client

class BidiSession(client.Session):
    def __init__(self,
                 host,
                 port,
                 url_prefix="/",
                 capabilities=None,
                 extension=None):
        """
        Add a capability of "webSocketUrl": True to enable
        Bidirectional connection in session creation.
        """
        self.websocket_transport = None
        capabilities = self._enable_websocket(capabilities)
        super().__init__(host, port, url_prefix, capabilities, extension)

    def _enable_websocket(self, caps):
        if caps:
            caps.setdefault("alwaysMatch", {}).update({"webSocketUrl": True})
        else:
            caps = {"alwaysMatch": {"webSocketUrl": True}}
        return caps

    def match(self, capabilities):
        """Expensive match to see if capabilities is the same as previously
        requested capabilities if websocket would be enabled.

        :return Boolean.
        """
        caps = copy.deepcopy(capabilities)
        caps = self._enable_websocket(caps)
        return super().match(caps)

    async def start(self):
        """Start a new WebDriver Bidirectional session
        with websocket connected.

        :return: Dictionary with `capabilities` and `sessionId`.
        """
        value = super().start()

        if not self.websocket_transport or not self.websocket_transport.open:
            self.websocket_transport = await websockets.connect(self.capabilities["webSocketUrl"])
        return value

    async def end(self):
        """Close websocket connection first before closing session.
        """
        if self.websocket_transport:
            await self.websocket_transport.close()
            self.websocket_transport = None
        super().end()
