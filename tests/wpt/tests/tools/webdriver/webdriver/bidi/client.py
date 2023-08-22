# mypy: allow-untyped-defs

import asyncio
from collections import defaultdict
from typing import Any, Awaitable, Callable, List, Optional, Mapping, MutableMapping
from urllib.parse import urljoin, urlparse

from . import modules
from .error import from_error_details
from .transport import get_running_loop, Transport


class BidiSession:
    """A WebDriver BiDi session.

    This is the main representation of a BiDi session and provides the
    interface for running commands in the session, and for attaching
    event handlers to the session. For example:

    async def on_log(method, data):
        print(data)

    session = BidiSession("ws://localhost:4445", capabilities)
    remove_listener = session.add_event_listener("log.entryAdded", on_log)
    await session.start()
    await session.subscribe("log.entryAdded")

    # Do some stuff with the session

    remove_listener()
    session.end()

    If the session id is provided it's assumed that the underlying
    WebDriver session was already created, and the WebSocket URL was
    taken from the new session response. If no session id is provided, it's
    assumed that a BiDi-only session should be created when start() is called.

    It can also be used as a context manager, with the WebSocket transport
    implictly being created when the context is entered, and closed when
    the context is exited.

    :param websocket_url: WebSockets URL on which to connect to the session.
                          This excludes any path component.
    :param session_id: String id of existing HTTP session
    :param capabilities: Capabilities response of existing session
    :param requested_capabilities: Dictionary representing the capabilities request.

    """

    def __init__(self,
                 websocket_url: str,
                 session_id: Optional[str] = None,
                 capabilities: Optional[Mapping[str, Any]] = None,
                 requested_capabilities: Optional[Mapping[str, Any]] = None):
        self.transport: Optional[Transport] = None

        # The full URL for a websocket looks like
        # ws://<host>:<port>/session when we're creating a session and
        # ws://<host>:<port>/session/<sessionid> when we're connecting to an existing session.
        # To be user friendly, handle the case where the class was created with either a
        # full URL including the path, and also the case where just a server url is passed in.
        parsed_url = urlparse(websocket_url)
        if parsed_url.path == "" or parsed_url.path == "/":
            if session_id is None:
                websocket_url = urljoin(websocket_url, "session")
            else:
                websocket_url = urljoin(websocket_url, f"session/{session_id}")
        else:
            if session_id is not None:
                if parsed_url.path != f"/session/{session_id}":
                    raise ValueError(f"WebSocket URL {session_id} doesn't match session id")
            else:
                if parsed_url.path != "/session":
                    raise ValueError(f"WebSocket URL {session_id} doesn't match session url")

        if session_id is None and capabilities is not None:
            raise ValueError("Tried to create BiDi-only session with existing capabilities")

        self.websocket_url = websocket_url
        self.requested_capabilities = requested_capabilities
        self.capabilities = capabilities
        self.session_id = session_id

        self.command_id = 0
        self.pending_commands: MutableMapping[int, "asyncio.Future[Any]"] = {}
        self.event_listeners: MutableMapping[
            Optional[str],
            List[Callable[[str, Mapping[str, Any]], Any]]
        ] = defaultdict(list)

        # Modules.
        # For each module, have a property representing that module
        self.browser = modules.Browser(self)
        self.browsing_context = modules.BrowsingContext(self)
        self.input = modules.Input(self)
        self.network = modules.Network(self)
        self.script = modules.Script(self)
        self.session = modules.Session(self)

    @property
    def event_loop(self):
        if self.transport:
            return self.transport.loop

        return None

    @classmethod
    def from_http(cls,
                  session_id: str,
                  capabilities: Mapping[str, Any]) -> "BidiSession":
        """Create a BiDi session from an existing HTTP session

        :param session_id: String id of the session
        :param capabilities: Capabilities returned in the New Session HTTP response."""
        websocket_url = capabilities.get("webSocketUrl")
        if websocket_url is None:
            raise ValueError("No webSocketUrl found in capabilities")
        if not isinstance(websocket_url, str):
            raise ValueError("webSocketUrl is not a string")
        return cls(websocket_url, session_id=session_id, capabilities=capabilities)

    @classmethod
    def bidi_only(cls,
                  websocket_url: str,
                  requested_capabilities: Optional[Mapping[str, Any]] = None) -> "BidiSession":
        """Create a BiDi session where there is no existing HTTP session

        :param webdocket_url: URL to the WebSocket server listening for BiDi connections
        :param requested_capabilities: Capabilities request for establishing the session."""
        return cls(websocket_url, requested_capabilities=requested_capabilities)

    async def __aenter__(self) -> "BidiSession":
        await self.start()
        return self

    async def __aexit__(self, *args: Any) -> None:
        await self.end()

    async def start_transport(self,
                              loop: Optional[asyncio.AbstractEventLoop] = None) -> None:
        if self.transport is None:
            if loop is None:
                loop = get_running_loop()

            self.transport = Transport(self.websocket_url, self.on_message, loop=loop)
            await self.transport.start()

    async def start(self,
                    loop: Optional[asyncio.AbstractEventLoop] = None) -> None:
        """Connect to the WebDriver BiDi remote via WebSockets"""

        await self.start_transport(loop)

        if self.session_id is None:
            self.session_id, self.capabilities = await self.session.new(  # type: ignore
                capabilities=self.requested_capabilities)

    async def send_command(
        self,
        method: str,
        params: Mapping[str, Any]
    ) -> Awaitable[Mapping[str, Any]]:
        """Send a command to the remote server"""
        # this isn't threadsafe
        self.command_id += 1
        command_id = self.command_id

        body = {
            "id": command_id,
            "method": method,
            "params": params
        }
        assert command_id not in self.pending_commands
        assert self.transport is not None
        self.pending_commands[command_id] = self.transport.loop.create_future()
        await self.transport.send(body)

        return self.pending_commands[command_id]

    async def on_message(self, data: Mapping[str, Any]) -> None:
        """Handle a message from the remote server"""
        if data["type"] in ["error", "success"]:
            # This is a command response or error
            future = self.pending_commands.get(data["id"])
            if future is None:
                raise ValueError(f"No pending command with id {data['id']}")
            if data["type"] == "success":
                assert isinstance(data["result"], dict)
                future.set_result(data["result"])
            else:
                assert isinstance(data["error"], str)
                assert isinstance(data["message"], str)
                exception = from_error_details(data["error"],
                                               data["message"],
                                               data.get("stacktrace"))
                future.set_exception(exception)
        elif data["type"] == "event":
            # This is an event
            assert isinstance(data["method"], str)
            assert isinstance(data["params"], dict)

            listeners = self.event_listeners.get(data["method"], [])
            if not listeners:
                listeners = self.event_listeners.get(None, [])
            for listener in listeners:
                await listener(data["method"], data["params"])
        else:
            raise ValueError(f"Unexpected message: {data!r}")

    async def end(self) -> None:
        """Close websocket connection."""
        assert self.transport is not None
        await self.transport.end()
        self.transport = None

    def add_event_listener(
        self,
        name: Optional[str],
        fn: Callable[[str, Mapping[str, Any]], Awaitable[Any]]
    ) -> Callable[[], None]:
        """Add a listener for the event with a given name.

        If name is None, the listener is called for all messages that are not otherwise
        handled.

        :param name: Name of event to listen for or None to register a default handler
        :param fn: Async callback function that receives event data

        :return: Function to remove the added listener
        """
        self.event_listeners[name].append(fn)

        return lambda: self.event_listeners[name].remove(fn)
