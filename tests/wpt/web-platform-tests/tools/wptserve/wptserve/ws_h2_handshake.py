"""This file provides the opening handshake processor for the Bootstrapping
WebSockets with HTTP/2 protocol (RFC 8441).

Specification:
https://tools.ietf.org/html/rfc8441
"""

from __future__ import absolute_import

from mod_pywebsocket import common
from mod_pywebsocket.stream import Stream
from mod_pywebsocket.stream import StreamOptions
from mod_pywebsocket import util
from six.moves import map
from six.moves import range

# TODO: We are using "private" methods of pywebsocket. We might want to
# refactor pywebsocket to expose those methods publicly. Also, _get_origin
# _check_version _set_protocol _parse_extensions and a large part of do_handshake
# are identical with hybi handshake. We need some refactoring to get remove that
# code duplication.
from mod_pywebsocket.extensions import get_extension_processor
from mod_pywebsocket.handshake._base import get_mandatory_header
from mod_pywebsocket.handshake._base import HandshakeException
from mod_pywebsocket.handshake._base import parse_token_list
from mod_pywebsocket.handshake._base import validate_mandatory_header
from mod_pywebsocket.handshake._base import validate_subprotocol
from mod_pywebsocket.handshake._base import VersionException

# Defining aliases for values used frequently.
_VERSION_LATEST = common.VERSION_HYBI_LATEST
_VERSION_LATEST_STRING = str(_VERSION_LATEST)
_SUPPORTED_VERSIONS = [
    _VERSION_LATEST,
]


def check_connect_method(request):
    if request.method != u'CONNECT':
        raise HandshakeException('Method is not CONNECT: %r' % request.method)


class WsH2Handshaker(object):
    def __init__(self, request, dispatcher):
        """Opening handshake processor for the WebSocket protocol (RFC 6455).

        :param request: mod_python request.

        :param dispatcher: Dispatcher (dispatch.Dispatcher).

        WsH2Handshaker will add attributes such as ws_resource during handshake.
        """

        self._logger = util.get_class_logger(self)

        self._request = request
        self._dispatcher = dispatcher

    def _get_extension_processors_requested(self):
        processors = []
        if self._request.ws_requested_extensions is not None:
            for extension_request in self._request.ws_requested_extensions:
                processor = get_extension_processor(extension_request)
                # Unknown extension requests are just ignored.
                if processor is not None:
                    processors.append(processor)
        return processors

    def do_handshake(self):
        self._request.ws_close_code = None
        self._request.ws_close_reason = None

        # Parsing.

        check_connect_method(self._request)

        validate_mandatory_header(self._request, ':protocol', 'websocket')

        self._request.ws_resource = self._request.uri

        get_mandatory_header(self._request, 'authority')

        self._request.ws_version = self._check_version()

        try:
            self._get_origin()
            self._set_protocol()
            self._parse_extensions()

            # Setup extension processors.
            self._request.ws_extension_processors = self._get_extension_processors_requested()

            # List of extra headers. The extra handshake handler may add header
            # data as name/value pairs to this list and pywebsocket appends
            # them to the WebSocket handshake.
            self._request.extra_headers = []

            # Extra handshake handler may modify/remove processors.
            self._dispatcher.do_extra_handshake(self._request)
            processors = [
                processor
                for processor in self._request.ws_extension_processors
                if processor is not None
            ]

            # Ask each processor if there are extensions on the request which
            # cannot co-exist. When processor decided other processors cannot
            # co-exist with it, the processor marks them (or itself) as
            # "inactive". The first extension processor has the right to
            # make the final call.
            for processor in reversed(processors):
                if processor.is_active():
                    processor.check_consistency_with_other_processors(
                        processors)
            processors = [
                processor for processor in processors if processor.is_active()
            ]

            accepted_extensions = []

            stream_options = StreamOptions()

            for index, processor in enumerate(processors):
                if not processor.is_active():
                    continue

                extension_response = processor.get_extension_response()
                if extension_response is None:
                    # Rejected.
                    continue

                accepted_extensions.append(extension_response)

                processor.setup_stream_options(stream_options)

                # Inactivate all of the following compression extensions.
                for j in range(index + 1, len(processors)):
                    processors[j].set_active(False)

            if len(accepted_extensions) > 0:
                self._request.ws_extensions = accepted_extensions
                self._logger.debug(
                    'Extensions accepted: %r',
                    list(
                        map(common.ExtensionParameter.name,
                            accepted_extensions)))
            else:
                self._request.ws_extensions = None

            self._request.ws_stream = self._create_stream(stream_options)

            if self._request.ws_requested_protocols is not None:
                if self._request.ws_protocol is None:
                    raise HandshakeException(
                        'do_extra_handshake must choose one subprotocol from '
                        'ws_requested_protocols and set it to ws_protocol')
                validate_subprotocol(self._request.ws_protocol)

                self._logger.debug('Subprotocol accepted: %r',
                                   self._request.ws_protocol)
            else:
                if self._request.ws_protocol is not None:
                    raise HandshakeException(
                        'ws_protocol must be None when the client didn\'t '
                        'request any subprotocol')

            self._prepare_handshake_response()
        except HandshakeException as e:
            if not e.status:
                # Fallback to 400 bad request by default.
                e.status = common.HTTP_STATUS_BAD_REQUEST
            raise e

    def _get_origin(self):
        origin = self._request.headers_in.get('origin')
        if origin is None:
            self._logger.debug('Client request does not have origin header')
        self._request.ws_origin = origin

    def _check_version(self):
        sec_websocket_version_header = 'sec-websocket-version'
        version = get_mandatory_header(self._request, sec_websocket_version_header)
        if version == _VERSION_LATEST_STRING:
            return _VERSION_LATEST

        if version.find(',') >= 0:
            raise HandshakeException(
                'Multiple versions (%r) are not allowed for header %s' %
                (version, sec_websocket_version_header),
                status=common.HTTP_STATUS_BAD_REQUEST)
        raise VersionException('Unsupported version %r for header %s' %
                               (version, sec_websocket_version_header),
                               supported_versions=', '.join(
                                   map(str, _SUPPORTED_VERSIONS)))

    def _set_protocol(self):
        self._request.ws_protocol = None

        protocol_header = self._request.headers_in.get('sec-websocket-protocol')

        if protocol_header is None:
            self._request.ws_requested_protocols = None
            return

        self._request.ws_requested_protocols = parse_token_list(
            protocol_header)
        self._logger.debug('Subprotocols requested: %r',
                           self._request.ws_requested_protocols)

    def _parse_extensions(self):
        extensions_header = self._request.headers_in.get('sec-websocket-extensions')
        if not extensions_header:
            self._request.ws_requested_extensions = None
            return

        try:
            self._request.ws_requested_extensions = common.parse_extensions(
                extensions_header)
        except common.ExtensionParsingException as e:
            raise HandshakeException(
                'Failed to parse sec-websocket-extensions header: %r' % e)

        self._logger.debug(
            'Extensions requested: %r',
            list(
                map(common.ExtensionParameter.name,
                    self._request.ws_requested_extensions)))

    def _create_stream(self, stream_options):
        return Stream(self._request, stream_options)

    def _prepare_handshake_response(self):
        self._request.status = 200

        self._request.headers_out['upgrade'] = common.WEBSOCKET_UPGRADE_TYPE
        self._request.headers_out['connection'] = common.UPGRADE_CONNECTION_TYPE

        if self._request.ws_protocol is not None:
            self._request.headers_out['sec-websocket-protocol'] = self._request.ws_protocol

        if (self._request.ws_extensions is not None and
                len(self._request.ws_extensions) != 0):
            self._request.headers_out['sec-websocket-extensions'] = common.format_extensions(self._request.ws_extensions)

        # Headers not specific for WebSocket
        for name, value in self._request.extra_headers:
            self._request.headers_out[name] = value
