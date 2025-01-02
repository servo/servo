#!/usr/bin/env python

import asyncio
import http
import http.cookies
import pathlib
import signal
import urllib.parse
import uuid

import websockets
from websockets.frames import CloseCode


# User accounts database

USERS = {}


def create_token(user, lifetime=1):
    """Create token for user and delete it once its lifetime is over."""
    token = uuid.uuid4().hex
    USERS[token] = user
    asyncio.get_running_loop().call_later(lifetime, USERS.pop, token)
    return token


def get_user(token):
    """Find user authenticated by token or return None."""
    return USERS.get(token)


# Utilities


def get_cookie(raw, key):
    cookie = http.cookies.SimpleCookie(raw)
    morsel = cookie.get(key)
    if morsel is not None:
        return morsel.value


def get_query_param(path, key):
    query = urllib.parse.urlparse(path).query
    params = urllib.parse.parse_qs(query)
    values = params.get(key, [])
    if len(values) == 1:
        return values[0]


# Main HTTP server

CONTENT_TYPES = {
    ".css": "text/css",
    ".html": "text/html; charset=utf-8",
    ".ico": "image/x-icon",
    ".js": "text/javascript",
}


async def serve_html(path, request_headers):
    user = get_query_param(path, "user")
    path = urllib.parse.urlparse(path).path
    if path == "/":
        if user is None:
            page = "index.html"
        else:
            page = "test.html"
    else:
        page = path[1:]

    try:
        template = pathlib.Path(__file__).with_name(page)
    except ValueError:
        pass
    else:
        if template.is_file():
            headers = {"Content-Type": CONTENT_TYPES[template.suffix]}
            body = template.read_bytes()
            if user is not None:
                token = create_token(user)
                body = body.replace(b"TOKEN", token.encode())
            return http.HTTPStatus.OK, headers, body

    return http.HTTPStatus.NOT_FOUND, {}, b"Not found\n"


async def noop_handler(websocket):
    pass


# Send credentials as the first message in the WebSocket connection


async def first_message_handler(websocket):
    token = await websocket.recv()
    user = get_user(token)
    if user is None:
        await websocket.close(CloseCode.INTERNAL_ERROR, "authentication failed")
        return

    await websocket.send(f"Hello {user}!")
    message = await websocket.recv()
    assert message == f"Goodbye {user}."


# Add credentials to the WebSocket URI in a query parameter


class QueryParamProtocol(websockets.WebSocketServerProtocol):
    async def process_request(self, path, headers):
        token = get_query_param(path, "token")
        if token is None:
            return http.HTTPStatus.UNAUTHORIZED, [], b"Missing token\n"

        user = get_user(token)
        if user is None:
            return http.HTTPStatus.UNAUTHORIZED, [], b"Invalid token\n"

        self.user = user


async def query_param_handler(websocket):
    user = websocket.user

    await websocket.send(f"Hello {user}!")
    message = await websocket.recv()
    assert message == f"Goodbye {user}."


# Set a cookie on the domain of the WebSocket URI


class CookieProtocol(websockets.WebSocketServerProtocol):
    async def process_request(self, path, headers):
        if "Upgrade" not in headers:
            template = pathlib.Path(__file__).with_name(path[1:])
            headers = {"Content-Type": CONTENT_TYPES[template.suffix]}
            body = template.read_bytes()
            return http.HTTPStatus.OK, headers, body

        token = get_cookie(headers.get("Cookie", ""), "token")
        if token is None:
            return http.HTTPStatus.UNAUTHORIZED, [], b"Missing token\n"

        user = get_user(token)
        if user is None:
            return http.HTTPStatus.UNAUTHORIZED, [], b"Invalid token\n"

        self.user = user


async def cookie_handler(websocket):
    user = websocket.user

    await websocket.send(f"Hello {user}!")
    message = await websocket.recv()
    assert message == f"Goodbye {user}."


# Adding credentials to the WebSocket URI in user information


class UserInfoProtocol(websockets.BasicAuthWebSocketServerProtocol):
    async def check_credentials(self, username, password):
        if username != "token":
            return False

        user = get_user(password)
        if user is None:
            return False

        self.user = user
        return True


async def user_info_handler(websocket):
    user = websocket.user

    await websocket.send(f"Hello {user}!")
    message = await websocket.recv()
    assert message == f"Goodbye {user}."


# Start all five servers


async def main():
    # Set the stop condition when receiving SIGINT or SIGTERM.
    loop = asyncio.get_running_loop()
    stop = loop.create_future()
    loop.add_signal_handler(signal.SIGINT, stop.set_result, None)
    loop.add_signal_handler(signal.SIGTERM, stop.set_result, None)

    async with websockets.serve(
        noop_handler,
        host="",
        port=8000,
        process_request=serve_html,
    ), websockets.serve(
        first_message_handler,
        host="",
        port=8001,
    ), websockets.serve(
        query_param_handler,
        host="",
        port=8002,
        create_protocol=QueryParamProtocol,
    ), websockets.serve(
        cookie_handler,
        host="",
        port=8003,
        create_protocol=CookieProtocol,
    ), websockets.serve(
        user_info_handler,
        host="",
        port=8004,
        create_protocol=UserInfoProtocol,
    ):
        print("Running on http://localhost:8000/")
        await stop
        print("\rExiting")


if __name__ == "__main__":
    asyncio.run(main())
