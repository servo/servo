import asyncio
import functools
import json
import logging
import sys
from collections import defaultdict
from typing import Any, Awaitable, Callable, Coroutine, List, Optional, Mapping, MutableMapping
from urllib.parse import urljoin, urlparse

import websockets

from .error import from_error_details

logger = logging.getLogger("webdriver.bidi")


def get_running_loop() -> asyncio.AbstractEventLoop:
    if sys.version_info >= (3, 7):
        return asyncio.get_running_loop()
    # Unlike the above, this will actually create an event loop
    # if there isn't one; hopefully running tests in Python >= 3.7
    # will allow us to catch any behaviour difference
    return asyncio.get_event_loop()


class BidiSession:
    """A WebDriver BiDi session.

    This is the main representation of a BiDi session and provides the
    interface for running commands in the session, and for attaching
    event handlers to the session. For example:

    async def on_log(data):
        print(data)

    session = BidiSession("ws://localhost:4445", capabilities)
    session.add_event_listener("log.entryAdded", on_log)
    await session.start()
    await session.subscribe("log.entryAdded")
    # Do some stuff with the session
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
                 requested_capabilities: Optional[Mapping[str, Any]] = None,
                 loop: Optional[asyncio.AbstractEventLoop] = None):
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
        self.event_listeners: MutableMapping[Optional[str], List[Callable[[str, Mapping[str, Any]], Any]]] = defaultdict(list)

        # Modules.
        # For each module, have a property representing that module
        self.session = Session(self)

        if loop is None:
            loop = get_running_loop()
        self.loop = loop

    @classmethod
    def from_http(cls,
                  session_id: str,
                  capabilities: Mapping[str, Any],
                  loop: Optional[asyncio.AbstractEventLoop] = None) -> "BidiSession":
        """Create a BiDi session from an existing HTTP session

        :param session_id: String id of the session
        :param capabilities: Capabilities returned in the New Session HTTP response."""
        websocket_url = capabilities.get("webSocketUrl")
        if websocket_url is None:
            raise ValueError("No webSocketUrl found in capabilities")
        if not isinstance(websocket_url, str):
            raise ValueError("webSocketUrl is not a string")
        return cls(websocket_url, session_id=session_id, capabilities=capabilities, loop=loop)

    @classmethod
    def bidi_only(cls,
                  websocket_url: str,
                  requested_capabilities: Optional[Mapping[str, Any]],
                  loop: Optional[asyncio.AbstractEventLoop] = None) -> "BidiSession":
        """Create a BiDi session where there is no existing HTTP session

        :param webdocket_url: URL to the WebSocket server listening for BiDi connections
        :param requested_capabilities: Capabilities request for establishing the session."""
        return cls(websocket_url, requested_capabilities=requested_capabilities, loop=loop)

    async def __aenter__(self) -> "BidiSession":
        await self.start()
        return self

    async def __aexit__(self, *args: Any) -> None:
        await self.end()

    async def start(self) -> None:
        """Connect to the WebDriver BiDi remote via WebSockets"""
        self.transport = Transport(self.websocket_url, self.on_message, loop=self.loop)

        if self.session_id is None:
            self.session_id, self.capabilities = await self.session.new(self.requested_capabilities)

        await self.transport.start()

    async def send_command(self, method: str, params: Mapping[str, Any]) -> Awaitable[Mapping[str, Any]]:
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
        self.pending_commands[command_id] = self.loop.create_future()
        assert self.transport is not None
        await self.transport.send(body)

        return self.pending_commands[command_id]

    async def on_message(self, data: Mapping[str, Any]) -> None:
        """Handle a message from the remote server"""
        if "id" in data:
            # This is a command response or error
            future = self.pending_commands.get(data["id"])
            if future is None:
                raise ValueError(f"No pending command with id {data['id']}")
            if "result" in data:
                future.set_result(data["result"])
            elif "error" in data and "message" in data:
                assert isinstance(data["error"], str)
                assert isinstance(data["message"], str)
                exception = from_error_details(data["error"],
                                               data["message"],
                                               data.get("stacktrace"))
                future.set_exception(exception)
            else:
                raise ValueError(f"Unexpected message: {data!r}")
        elif "method" in data and "params" in data:
            # This is an event
            method = data["method"]
            listeners = self.event_listeners.get(method, [])
            if not listeners:
                listeners = self.event_listeners.get(None, [])
            for listener in listeners:
                await listener(method, data["params"])
        else:
            raise ValueError(f"Unexpected message: {data!r}")

    async def end(self) -> None:
        """Close websocket connection."""
        assert self.transport is not None
        await self.transport.end()

    def add_event_listener(self,
                           name: Optional[str],
                           fn: Callable[[str, Mapping[str, Any]], Awaitable[Any]]) -> None:
        """Add a listener for the event with a given name.

        If name is None, the listener is called for all messages that are not otherwise
        handled.

        :param name: Name of event to listen for or None to register a default handler
        :param fn: Async callback function that receives event data
        """
        self.event_listeners[name].append(fn)


class Transport:
    """Low level message handler for the WebSockets connection"""
    def __init__(self, url: str,
                 msg_handler: Callable[[Mapping[str, Any]], Coroutine[Any, Any, None]],
                 loop: Optional[asyncio.AbstractEventLoop] = None):
        self.url = url
        self.connection: Optional[websockets.WebSocketClientProtocol] = None
        self.msg_handler = msg_handler
        self.send_buf: List[Mapping[str, Any]] = []

        if loop is None:
            loop = get_running_loop()
        self.loop = loop

        self.read_message_task: Optional[asyncio.Task[Any]] = None

    async def start(self) -> None:
        self.connection = await websockets.client.connect(self.url)
        self.read_message_task = self.loop.create_task(self.read_messages())

        for msg in self.send_buf:
            await self._send(self.connection, msg)

    async def send(self, data: Mapping[str, Any]) -> None:
        if self.connection is not None:
            await self._send(self.connection, data)
        else:
            self.send_buf.append(data)

    @staticmethod
    async def _send(connection: websockets.WebSocketClientProtocol, data: Mapping[str, Any]) -> None:
        msg = json.dumps(data)
        logger.debug("→ %s", msg)
        await connection.send(msg)

    async def handle(self, msg: str) -> None:
        logger.debug("← %s", msg)
        data = json.loads(msg)
        await self.msg_handler(data)

    async def end(self) -> None:
        if self.connection:
            await self.connection.close()
            self.connection = None

    async def read_messages(self) -> None:
        assert self.connection is not None
        async for msg in self.connection:
            if not isinstance(msg, str):
                raise ValueError("Got a binary message")
            await self.handle(msg)


class command:
    """Decorator for implementing bidi commands

    Implementing a command involves specifying an async function that
    builds the parameters to the command. The decorator arranges those
    parameters to be turned into a send_command call, using the class
    and method names to determine the method in the call.

    Commands decorated in this way don't return a future, but await
    the actual response. In some cases it can be useful to
    post-process this response before returning it to the client. This
    can be done by specifying a second decorated method like
    @command_name.result. That method will then be called once the
    result of the original command is known, and the return value of
    the method used as the response of the command.

    So for an example, if we had a command test.testMethod, which
    returned a result which we want to convert to a TestResult type,
    the implementation might look like:

    class Test(BidiModule):
        @command
        def test_method(self, test_data=None):
            return {"testData": test_data}

       @test_method.result
       def convert_test_method_result(self, result):
           return TestData(**result)
    """

    def __init__(self, fn: Callable[..., Mapping[str, Any]]):
        self.params_fn = fn
        self.result_fn: Optional[Callable[..., Any]] = None

    def result(self, fn: Callable[[Any, MutableMapping[str, Any]], Mapping[str, Any]]) -> None:
        self.result_fn = fn

    def __set_name__(self, owner: Any, name: str) -> None:
        # This is called when the class is created
        # see https://docs.python.org/3/reference/datamodel.html#object.__set_name__
        params_fn = self.params_fn
        result_fn = self.result_fn

        @functools.wraps(params_fn)
        async def inner(self: Any, **kwargs: Any) -> Any:
            params = params_fn(self, **kwargs)

            # Convert the classname and the method name to a bidi command name
            mod_name = owner.__name__.lower()
            if hasattr(owner, "prefix"):
                mod_name = f"{owner.prefix}:{mod_name}"
            cmd_name = f"{mod_name}.{to_camelcase(name)}"

            future = await self.session.send_command(cmd_name, params)
            result = await future

            if result_fn is not None:
                # Convert the result if we have a conversion function defined
                result = result_fn(self, result)
            return result

        # Overwrite the method on the owner class with the wrapper
        setattr(owner, name, inner)

    def __call__(*args: Any, **kwargs: Any) -> Awaitable[Any]:
        # This isn't really used, but mypy doesn't understand __set_name__
        pass


def to_camelcase(name: str) -> str:
    """Convert a python style method name foo_bar to a BiDi command name fooBar"""
    parts = name.split("_")
    parts[0] = parts[0].lower()
    for i in range(1, len(parts)):
        parts[i] = parts[i].title()
    return "".join(parts)


class BidiModule:
    def __init__(self, session: BidiSession):
        self.session = session


class Session(BidiModule):
    @command
    def new(self, capabilities: Mapping[str, Any]) -> Mapping[str, Mapping[str, Any]]:
        return {"capabilities": capabilities}

    @new.result
    def _new(self, result: Mapping[str, Any]) -> Any:
        return result.get("session_id"), result.get("capabilities", {})

    @command
    def subscribe(self,
                  events: List[str],
                  contexts: Optional[List[str]] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"events": events}
        if contexts is not None:
            params["contexts"] = contexts
        return params

    @command
    def unsubscribe(self,
                    events: Optional[List[str]] = None,
                    contexts: Optional[List[str]] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {"events": events if events is not None else []}
        if contexts is not None:
            params["contexts"] = contexts
        return params


class Test(BidiModule):
    """Very temporary module that does nothing, except demonstrate a vendor prefix and
    provide a way to work with Gecko's current skeleton implementation."""

    prefix = "moz"

    @command
    def test_method(self, **kwargs):
        return kwargs
