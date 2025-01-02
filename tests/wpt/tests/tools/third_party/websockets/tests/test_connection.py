from websockets.protocol import Protocol

from .utils import DeprecationTestCase


class BackwardsCompatibilityTests(DeprecationTestCase):
    def test_connection_class(self):
        with self.assertDeprecationWarning(
            "websockets.connection was renamed to websockets.protocol "
            "and Connection was renamed to Protocol"
        ):
            from websockets.connection import Connection

        self.assertIs(Connection, Protocol)
