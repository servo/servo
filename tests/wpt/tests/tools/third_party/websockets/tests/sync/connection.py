import contextlib
import time

from websockets.sync.connection import Connection


class InterceptingConnection(Connection):
    """
    Connection subclass that can intercept outgoing packets.

    By interfacing with this connection, you can simulate network conditions
    affecting what the component being tested receives during a test.

    """

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.socket = InterceptingSocket(self.socket)

    @contextlib.contextmanager
    def delay_frames_sent(self, delay):
        """
        Add a delay before sending frames.

        Delays cumulate: they're added before every frame or before EOF.

        """
        assert self.socket.delay_sendall is None
        self.socket.delay_sendall = delay
        try:
            yield
        finally:
            self.socket.delay_sendall = None

    @contextlib.contextmanager
    def delay_eof_sent(self, delay):
        """
        Add a delay before sending EOF.

        Delays cumulate: they're added before every frame or before EOF.

        """
        assert self.socket.delay_shutdown is None
        self.socket.delay_shutdown = delay
        try:
            yield
        finally:
            self.socket.delay_shutdown = None

    @contextlib.contextmanager
    def drop_frames_sent(self):
        """
        Prevent frames from being sent.

        Since TCP is reliable, sending frames or EOF afterwards is unrealistic.

        """
        assert not self.socket.drop_sendall
        self.socket.drop_sendall = True
        try:
            yield
        finally:
            self.socket.drop_sendall = False

    @contextlib.contextmanager
    def drop_eof_sent(self):
        """
        Prevent EOF from being sent.

        Since TCP is reliable, sending frames or EOF afterwards is unrealistic.

        """
        assert not self.socket.drop_shutdown
        self.socket.drop_shutdown = True
        try:
            yield
        finally:
            self.socket.drop_shutdown = False


class InterceptingSocket:
    """
    Socket wrapper that intercepts calls to sendall and shutdown.

    This is coupled to the implementation, which relies on these two methods.

    """

    def __init__(self, socket):
        self.socket = socket
        self.delay_sendall = None
        self.delay_shutdown = None
        self.drop_sendall = False
        self.drop_shutdown = False

    def __getattr__(self, name):
        return getattr(self.socket, name)

    def sendall(self, bytes, flags=0):
        if self.delay_sendall is not None:
            time.sleep(self.delay_sendall)
        if not self.drop_sendall:
            self.socket.sendall(bytes, flags)

    def shutdown(self, how):
        if self.delay_shutdown is not None:
            time.sleep(self.delay_shutdown)
        if not self.drop_shutdown:
            self.socket.shutdown(how)
