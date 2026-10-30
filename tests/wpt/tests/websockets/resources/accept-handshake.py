from pywebsocket3.handshake.hybi import compute_accept


def main(request, response):
    accept = compute_accept(request.headers.get(b"Sec-WebSocket-Key"))
    output =  b"HTTP/1.1 101 Switching Protocols\r\n"
    output += b"Upgrade: websocket\r\n"
    output += b"Connection: Upgrade\r\n"
    output += b"Sec-WebSocket-Accept: " + accept + b"\r\n"
    output += b"\r\n"
    response.writer.write(output)
    # Leave the connection open because closing it right after the handshake
    # can prevent the open event from firing.
