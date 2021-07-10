"""
:mod:`websockets.auth` provides HTTP Basic Authentication according to
:rfc:`7235` and :rfc:`7617`.

"""


import functools
import http
from typing import Any, Awaitable, Callable, Iterable, Optional, Tuple, Type, Union

from .exceptions import InvalidHeader
from .headers import build_www_authenticate_basic, parse_authorization_basic
from .http import Headers
from .server import HTTPResponse, WebSocketServerProtocol


__all__ = ["BasicAuthWebSocketServerProtocol", "basic_auth_protocol_factory"]

Credentials = Tuple[str, str]


def is_credentials(value: Any) -> bool:
    try:
        username, password = value
    except (TypeError, ValueError):
        return False
    else:
        return isinstance(username, str) and isinstance(password, str)


class BasicAuthWebSocketServerProtocol(WebSocketServerProtocol):
    """
    WebSocket server protocol that enforces HTTP Basic Auth.

    """

    def __init__(
        self,
        *args: Any,
        realm: str,
        check_credentials: Callable[[str, str], Awaitable[bool]],
        **kwargs: Any,
    ) -> None:
        self.realm = realm
        self.check_credentials = check_credentials
        super().__init__(*args, **kwargs)

    async def process_request(
        self, path: str, request_headers: Headers
    ) -> Optional[HTTPResponse]:
        """
        Check HTTP Basic Auth and return a HTTP 401 or 403 response if needed.

        If authentication succeeds, the username of the authenticated user is
        stored in the ``username`` attribute.

        """
        try:
            authorization = request_headers["Authorization"]
        except KeyError:
            return (
                http.HTTPStatus.UNAUTHORIZED,
                [("WWW-Authenticate", build_www_authenticate_basic(self.realm))],
                b"Missing credentials\n",
            )

        try:
            username, password = parse_authorization_basic(authorization)
        except InvalidHeader:
            return (
                http.HTTPStatus.UNAUTHORIZED,
                [("WWW-Authenticate", build_www_authenticate_basic(self.realm))],
                b"Unsupported credentials\n",
            )

        if not await self.check_credentials(username, password):
            return (
                http.HTTPStatus.UNAUTHORIZED,
                [("WWW-Authenticate", build_www_authenticate_basic(self.realm))],
                b"Invalid credentials\n",
            )

        self.username = username

        return await super().process_request(path, request_headers)


def basic_auth_protocol_factory(
    realm: str,
    credentials: Optional[Union[Credentials, Iterable[Credentials]]] = None,
    check_credentials: Optional[Callable[[str, str], Awaitable[bool]]] = None,
    create_protocol: Type[
        BasicAuthWebSocketServerProtocol
    ] = BasicAuthWebSocketServerProtocol,
) -> Callable[[Any], BasicAuthWebSocketServerProtocol]:
    """
    Protocol factory that enforces HTTP Basic Auth.

    ``basic_auth_protocol_factory`` is designed to integrate with
    :func:`~websockets.server.serve` like this::

        websockets.serve(
            ...,
            create_protocol=websockets.basic_auth_protocol_factory(
                realm="my dev server",
                credentials=("hello", "iloveyou"),
            )
        )

    ``realm`` indicates the scope of protection. It should contain only ASCII
    characters because the encoding of non-ASCII characters is undefined.
    Refer to section 2.2 of :rfc:`7235` for details.

    ``credentials`` defines hard coded authorized credentials. It can be a
    ``(username, password)`` pair or a list of such pairs.

    ``check_credentials`` defines a coroutine that checks whether credentials
    are authorized. This coroutine receives ``username`` and ``password``
    arguments and returns a :class:`bool`.

    One of ``credentials`` or ``check_credentials`` must be provided but not
    both.

    By default, ``basic_auth_protocol_factory`` creates a factory for building
    :class:`BasicAuthWebSocketServerProtocol` instances. You can override this
    with the ``create_protocol`` parameter.

    :param realm: scope of protection
    :param credentials: hard coded credentials
    :param check_credentials: coroutine that verifies credentials
    :raises TypeError: if the credentials argument has the wrong type

    """
    if (credentials is None) == (check_credentials is None):
        raise TypeError("provide either credentials or check_credentials")

    if credentials is not None:
        if is_credentials(credentials):

            async def check_credentials(username: str, password: str) -> bool:
                return (username, password) == credentials

        elif isinstance(credentials, Iterable):
            credentials_list = list(credentials)
            if all(is_credentials(item) for item in credentials_list):
                credentials_dict = dict(credentials_list)

                async def check_credentials(username: str, password: str) -> bool:
                    return credentials_dict.get(username) == password

            else:
                raise TypeError(f"invalid credentials argument: {credentials}")

        else:
            raise TypeError(f"invalid credentials argument: {credentials}")

    return functools.partial(
        create_protocol, realm=realm, check_credentials=check_credentials
    )
