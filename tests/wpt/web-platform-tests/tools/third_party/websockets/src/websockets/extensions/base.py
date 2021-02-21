"""
:mod:`websockets.extensions.base` defines abstract classes for implementing
extensions.

See `section 9 of RFC 6455`_.

.. _section 9 of RFC 6455: http://tools.ietf.org/html/rfc6455#section-9

"""

from typing import List, Optional, Sequence, Tuple

from ..framing import Frame
from ..typing import ExtensionName, ExtensionParameter


__all__ = ["Extension", "ClientExtensionFactory", "ServerExtensionFactory"]


class Extension:
    """
    Abstract class for extensions.

    """

    @property
    def name(self) -> ExtensionName:
        """
        Extension identifier.

        """

    def decode(self, frame: Frame, *, max_size: Optional[int] = None) -> Frame:
        """
        Decode an incoming frame.

        :param frame: incoming frame
        :param max_size: maximum payload size in bytes

        """

    def encode(self, frame: Frame) -> Frame:
        """
        Encode an outgoing frame.

        :param frame: outgoing frame

        """


class ClientExtensionFactory:
    """
    Abstract class for client-side extension factories.

    """

    @property
    def name(self) -> ExtensionName:
        """
        Extension identifier.

        """

    def get_request_params(self) -> List[ExtensionParameter]:
        """
        Build request parameters.

        Return a list of ``(name, value)`` pairs.

        """

    def process_response_params(
        self,
        params: Sequence[ExtensionParameter],
        accepted_extensions: Sequence[Extension],
    ) -> Extension:
        """
        Process response parameters received from the server.

        :param params: list of ``(name, value)`` pairs.
        :param accepted_extensions: list of previously accepted extensions.
        :raises ~websockets.exceptions.NegotiationError: if parameters aren't
            acceptable

        """


class ServerExtensionFactory:
    """
    Abstract class for server-side extension factories.

    """

    @property
    def name(self) -> ExtensionName:
        """
        Extension identifier.

        """

    def process_request_params(
        self,
        params: Sequence[ExtensionParameter],
        accepted_extensions: Sequence[Extension],
    ) -> Tuple[List[ExtensionParameter], Extension]:
        """
        Process request parameters received from the client.

        To accept the offer, return a 2-uple containing:

        - response parameters: a list of ``(name, value)`` pairs
        - an extension: an instance of a subclass of :class:`Extension`

        :param params: list of ``(name, value)`` pairs.
        :param accepted_extensions: list of previously accepted extensions.
        :raises ~websockets.exceptions.NegotiationError: to reject the offer,
            if parameters aren't acceptable

        """
