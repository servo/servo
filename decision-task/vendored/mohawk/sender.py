import logging

from .base import default_ts_skew_in_seconds, HawkAuthority, Resource
from .util import (calculate_mac,
                   parse_authorization_header,
                   validate_credentials)

__all__ = ['Sender']
log = logging.getLogger(__name__)


class Sender(HawkAuthority):
    """
    A Hawk authority that will emit requests and verify responses.

    :param credentials: Dict of credentials with keys ``id``, ``key``,
                        and ``algorithm``. See :ref:`usage` for an example.
    :type credentials: dict

    :param url: Absolute URL of the request.
    :type url: str

    :param method: Method of the request. E.G. POST, GET
    :type method: str

    :param content=None: Byte string of request body.
    :type content=None: str

    :param content_type=None: content-type header value for request.
    :type content_type=None: str

    :param always_hash_content=True:
        When True, ``content`` and ``content_type`` cannot be None.
        Read :ref:`skipping-content-checks` to learn more.
    :type always_hash_content=True: bool

    :param nonce=None:
        A string that when coupled with the timestamp will
        uniquely identify this request to prevent replays.
        If None, a nonce will be generated for you.
    :type nonce=None: str

    :param ext=None:
        An external `Hawk`_ string. If not None, this value will be signed
        so that the receiver can trust it.
    :type ext=None: str

    :param app=None:
        A `Hawk`_ application string. If not None, this value will be signed
        so that the receiver can trust it.
    :type app=None: str

    :param dlg=None:
        A `Hawk`_ delegation string. If not None, this value will be signed
        so that the receiver can trust it.
    :type dlg=None: str

    :param seen_nonce=None:
        A callable that returns True if a nonce has been seen.
        See :ref:`nonce` for details.
    :type seen_nonce=None: callable

    .. _`Hawk`: https://github.com/hueniverse/hawk
    """
    #: Value suitable for an ``Authorization`` header.
    request_header = None

    def __init__(self, credentials,
                 url,
                 method,
                 content=None,
                 content_type=None,
                 always_hash_content=True,
                 nonce=None,
                 ext=None,
                 app=None,
                 dlg=None,
                 seen_nonce=None,
                 # For easier testing:
                 _timestamp=None):

        self.reconfigure(credentials)
        self.request_header = None
        self.seen_nonce = seen_nonce

        log.debug('generating request header')
        self.req_resource = Resource(url=url,
                                     credentials=self.credentials,
                                     ext=ext,
                                     app=app,
                                     dlg=dlg,
                                     nonce=nonce,
                                     method=method,
                                     content=content,
                                     always_hash_content=always_hash_content,
                                     timestamp=_timestamp,
                                     content_type=content_type)

        mac = calculate_mac('header', self.req_resource,
                            self.req_resource.gen_content_hash())
        self.request_header = self._make_header(self.req_resource, mac)

    def accept_response(self,
                        response_header,
                        content=None,
                        content_type=None,
                        accept_untrusted_content=False,
                        localtime_offset_in_seconds=0,
                        timestamp_skew_in_seconds=default_ts_skew_in_seconds,
                        **auth_kw):
        """
        Accept a response to this request.

        :param response_header:
            A `Hawk`_ ``Server-Authorization`` header
            such as one created by :class:`mohawk.Receiver`.
        :type response_header: str

        :param content=None: Byte string of the response body received.
        :type content=None: str

        :param content_type=None:
            Content-Type header value of the response received.
        :type content_type=None: str

        :param accept_untrusted_content=False:
            When True, allow responses that do not hash their content or
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
        log.debug('accepting response {header}'
                  .format(header=response_header))

        parsed_header = parse_authorization_header(response_header)

        resource = Resource(ext=parsed_header.get('ext', None),
                            content=content,
                            content_type=content_type,
                            # The following response attributes are
                            # in reference to the original request,
                            # not to the reponse header:
                            timestamp=self.req_resource.timestamp,
                            nonce=self.req_resource.nonce,
                            url=self.req_resource.url,
                            method=self.req_resource.method,
                            app=self.req_resource.app,
                            dlg=self.req_resource.dlg,
                            credentials=self.credentials,
                            seen_nonce=self.seen_nonce)

        self._authorize(
            'response', parsed_header, resource,
            # Per Node lib, a responder macs the *sender's* timestamp.
            # It does not create its own timestamp.
            # I suppose a slow response could time out here. Maybe only check
            # mac failures, not timeouts?
            their_timestamp=resource.timestamp,
            timestamp_skew_in_seconds=timestamp_skew_in_seconds,
            localtime_offset_in_seconds=localtime_offset_in_seconds,
            accept_untrusted_content=accept_untrusted_content,
            **auth_kw)

    def reconfigure(self, credentials):
        validate_credentials(credentials)
        self.credentials = credentials
