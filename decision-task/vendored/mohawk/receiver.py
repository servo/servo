import logging
import sys

from .base import default_ts_skew_in_seconds, HawkAuthority, Resource
from .exc import CredentialsLookupError, MissingAuthorization
from .util import (calculate_mac,
                   parse_authorization_header,
                   validate_credentials)

__all__ = ['Receiver']
log = logging.getLogger(__name__)


class Receiver(HawkAuthority):
    """
    A Hawk authority that will receive and respond to requests.

    :param credentials_map:
        Callable to look up the credentials dict by sender ID.
        The credentials dict must have the keys:
        ``id``, ``key``, and ``algorithm``.
        See :ref:`receiving-request` for an example.
    :type credentials_map: callable

    :param request_header:
        A `Hawk`_ ``Authorization`` header
        such as one created by :class:`mohawk.Sender`.
    :type request_header: str

    :param url: Absolute URL of the request.
    :type url: str

    :param method: Method of the request. E.G. POST, GET
    :type method: str

    :param content=None: Byte string of request body.
    :type content=None: str

    :param content_type=None: content-type header value for request.
    :type content_type=None: str

    :param accept_untrusted_content=False:
        When True, allow requests that do not hash their content or
        allow None type ``content`` and ``content_type``
        arguments. Read :ref:`skipping-content-checks`
        to learn more.
    :type accept_untrusted_content=False: bool

    :param localtime_offset_in_seconds=0:
        Seconds to add to local time in case it's out of sync.
    :type localtime_offset_in_seconds=0: float

    :param timestamp_skew_in_seconds=60:
        Max seconds until a message expires. Upon expiry,
        :class:`mohawk.exc.TokenExpired` is raised.
    :type timestamp_skew_in_seconds=60: float

    .. _`Hawk`: https://github.com/hueniverse/hawk
    """
    #: Value suitable for a ``Server-Authorization`` header.
    response_header = None

    def __init__(self,
                 credentials_map,
                 request_header,
                 url,
                 method,
                 content=None,
                 content_type=None,
                 seen_nonce=None,
                 localtime_offset_in_seconds=0,
                 accept_untrusted_content=False,
                 timestamp_skew_in_seconds=default_ts_skew_in_seconds,
                 **auth_kw):

        self.response_header = None  # make into property that can raise exc?
        self.credentials_map = credentials_map
        self.seen_nonce = seen_nonce

        log.debug('accepting request {header}'.format(header=request_header))

        if not request_header:
            raise MissingAuthorization()

        parsed_header = parse_authorization_header(request_header)

        try:
            credentials = self.credentials_map(parsed_header['id'])
        except LookupError:
            etype, val, tb = sys.exc_info()
            log.debug('Catching {etype}: {val}'.format(etype=etype, val=val))
            raise CredentialsLookupError(
                'Could not find credentials for ID {0}'
                .format(parsed_header['id']))
        validate_credentials(credentials)

        resource = Resource(url=url,
                            method=method,
                            ext=parsed_header.get('ext', None),
                            app=parsed_header.get('app', None),
                            dlg=parsed_header.get('dlg', None),
                            credentials=credentials,
                            nonce=parsed_header['nonce'],
                            seen_nonce=self.seen_nonce,
                            content=content,
                            timestamp=parsed_header['ts'],
                            content_type=content_type)

        self._authorize(
            'header', parsed_header, resource,
            timestamp_skew_in_seconds=timestamp_skew_in_seconds,
            localtime_offset_in_seconds=localtime_offset_in_seconds,
            accept_untrusted_content=accept_untrusted_content,
            **auth_kw)

        # Now that we verified an incoming request, we can re-use some of its
        # properties to build our response header.

        self.parsed_header = parsed_header
        self.resource = resource

    def respond(self,
                content=None,
                content_type=None,
                always_hash_content=True,
                ext=None):
        """
        Respond to the request.

        This generates the :attr:`mohawk.Receiver.response_header`
        attribute.

        :param content=None: Byte string of response body that will be sent.
        :type content=None: str

        :param content_type=None: content-type header value for response.
        :type content_type=None: str

        :param always_hash_content=True:
            When True, ``content`` and ``content_type`` cannot be None.
            Read :ref:`skipping-content-checks` to learn more.
        :type always_hash_content=True: bool

        :param ext=None:
            An external `Hawk`_ string. If not None, this value will be
            signed so that the sender can trust it.
        :type ext=None: str

        .. _`Hawk`: https://github.com/hueniverse/hawk
        """

        log.debug('generating response header')

        resource = Resource(url=self.resource.url,
                            credentials=self.resource.credentials,
                            ext=ext,
                            app=self.parsed_header.get('app', None),
                            dlg=self.parsed_header.get('dlg', None),
                            method=self.resource.method,
                            content=content,
                            content_type=content_type,
                            always_hash_content=always_hash_content,
                            nonce=self.parsed_header['nonce'],
                            timestamp=self.parsed_header['ts'])

        mac = calculate_mac('response', resource, resource.gen_content_hash())

        self.response_header = self._make_header(resource, mac,
                                                 additional_keys=['ext'])
        return self.response_header
