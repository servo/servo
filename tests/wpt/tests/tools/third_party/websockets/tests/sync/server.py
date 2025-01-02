import contextlib
import ssl
import threading

from websockets.sync.server import *

from ..utils import CERTIFICATE


SERVER_CONTEXT = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
SERVER_CONTEXT.load_cert_chain(CERTIFICATE)

# Work around https://github.com/openssl/openssl/issues/7967

# This bug causes connect() to hang in tests for the client. Including this
# workaround acknowledges that the issue could happen outside of the test suite.

# It shouldn't happen too often, or else OpenSSL 1.1.1 would be unusable. If it
# happens, we can look for a library-level fix, but it won't be easy.

SERVER_CONTEXT.num_tickets = 0


def crash(ws):
    raise RuntimeError


def do_nothing(ws):
    pass


def eval_shell(ws):
    for expr in ws:
        value = eval(expr)
        ws.send(str(value))


class EvalShellMixin:
    def assertEval(self, client, expr, value):
        client.send(expr)
        self.assertEqual(client.recv(), value)


@contextlib.contextmanager
def run_server(ws_handler=eval_shell, host="localhost", port=0, **kwargs):
    with serve(ws_handler, host, port, **kwargs) as server:
        thread = threading.Thread(target=server.serve_forever)
        thread.start()
        try:
            yield server
        finally:
            server.shutdown()
            thread.join()


@contextlib.contextmanager
def run_unix_server(path, ws_handler=eval_shell, **kwargs):
    with unix_serve(ws_handler, path, **kwargs) as server:
        thread = threading.Thread(target=server.serve_forever)
        thread.start()
        try:
            yield server
        finally:
            server.shutdown()
            thread.join()
