import logging
import math
import pprint

import six
from six.moves.urllib.parse import urlparse

from .exc import (AlreadyProcessed,
                  MacMismatch,
                  MisComputedContentHash,
                  TokenExpired)
from .util import (calculate_mac,
                   calculate_payload_hash,
                   calculate_ts_mac,
                   prepare_header_val,
                   random_string,
                   strings_match,
                   utc_now)

default_ts_skew_in_seconds = 60
log = logging.getLogger(__name__)


class HawkAuthority:

    def _authorize(self, mac_type, parsed_header, resource,
                   their_timestamp=None,
                   timestamp_skew_in_seconds=default_ts_skew_in_seconds,
                   localtime_offset_in_seconds=0,
                   accept_untrusted_content=False):

        now = utc_now(offset_in_seconds=localtime_offset_in_seconds)

        their_hash = parsed_header.get('hash', '')
        their_mac = parsed_header.get('mac', '')
        mac = calculate_mac(mac_type, resource, their_hash)
        if not strings_match(mac, their_mac):
            raise MacMismatch('MACs do not match; ours: {ours}; '
                              'theirs: {theirs}'
                              .format(ours=mac, theirs=their_mac))

        if 'hash' not in parsed_header and accept_untrusted_content:
            # The request did not hash its content.
            log.debug('NOT calculating/verifiying payload hash '
                      '(no hash in header)')
            check_hash = False
            content_hash = None
        else:
            check_hash = True
            content_hash = resource.gen_content_hash()

        if check_hash and not their_hash:
            log.info('request unexpectedly did not hash its content')

        if check_hash:
            if not strings_match(content_hash, their_hash):
                # The hash declared in the header is incorrect.
                # Content could have been tampered with.
                log.debug('mismatched content: {content}'
                          .format(content=repr(resource.content)))
                log.debug('mismatched content-type: {typ}'
                          .format(typ=repr(resource.content_type)))
                raise MisComputedContentHash(
                    'Our hash {ours} ({algo}) did not '
                    'match theirs {theirs}'
                    .format(ours=content_hash,
                            theirs=their_hash,
                            algo=resource.credentials['algorithm']))

        if resource.seen_nonce:
            if resource.seen_nonce(resource.credentials['id'],
                                   parsed_header['nonce'],
                                   parsed_header['ts']):
                raise AlreadyProcessed('Nonce {nonce} with timestamp {ts} '
                                       'has already been processed for {id}'
                                       .format(nonce=parsed_header['nonce'],
                                               ts=parsed_header['ts'],
                                               id=resource.credentials['id']))
        else:
            log.warn('seen_nonce was None; not checking nonce. '
                     'You may be vulnerable to replay attacks')

        their_ts = int(their_timestamp or parsed_header['ts'])

        if math.fabs(their_ts - now) > timestamp_skew_in_seconds:
            message = ('token with UTC timestamp {ts} has expired; '
                       'it was compared to {now}'
                       .format(ts=their_ts, now=now))
            tsm = calculate_ts_mac(now, resource.credentials)
            if isinstance(tsm, six.binary_type):
                tsm = tsm.decode('ascii')
            www_authenticate = ('Hawk ts="{ts}", tsm="{tsm}", error="{error}"'
                                .format(ts=now, tsm=tsm, error=message))
            raise TokenExpired(message,
                               localtime_in_seconds=now,
                               www_authenticate=www_authenticate)

        log.debug('authorized OK')

    def _make_header(self, resource, mac, additional_keys=None):
        keys = additional_keys
        if not keys:
            # These are the default header keys that you'd send with a
            # request header. Response headers are odd because they
            # exclude a bunch of keys.
            keys = ('id', 'ts', 'nonce', 'ext', 'app', 'dlg')

        header = u'Hawk mac="{mac}"'.format(mac=prepare_header_val(mac))

        if resource.content_hash:
            header = u'{header}, hash="{hash}"'.format(
                header=header,
                hash=prepare_header_val(resource.content_hash))

        if 'id' in keys:
            header = u'{header}, id="{id}"'.format(
                header=header,
                id=prepare_header_val(resource.credentials['id']))

        if 'ts' in keys:
            header = u'{header}, ts="{ts}"'.format(
                header=header, ts=prepare_header_val(resource.timestamp))

        if 'nonce' in keys:
            header = u'{header}, nonce="{nonce}"'.format(
                header=header, nonce=prepare_header_val(resource.nonce))

        # These are optional so we need to check if they have values first.

        if 'ext' in keys and resource.ext:
            header = u'{header}, ext="{ext}"'.format(
                header=header, ext=prepare_header_val(resource.ext))

        if 'app' in keys and resource.app:
            header = u'{header}, app="{app}"'.format(
                header=header, app=prepare_header_val(resource.app))

        if 'dlg' in keys and resource.dlg:
            header = u'{header}, dlg="{dlg}"'.format(
                header=header, dlg=prepare_header_val(resource.dlg))

        log.debug('Hawk header for URL={url} method={method}: {header}'
                  .format(url=resource.url, method=resource.method,
                          header=header))
        return header


class Resource:
    """
    Normalized request/response resource.
    """

    def __init__(self, **kw):
        self.credentials = kw.pop('credentials')
        self.method = kw.pop('method').upper()
        self.content = kw.pop('content', None)
        self.content_type = kw.pop('content_type', None)
        self.always_hash_content = kw.pop('always_hash_content', True)
        self.ext = kw.pop('ext', None)
        self.app = kw.pop('app', None)
        self.dlg = kw.pop('dlg', None)

        self.timestamp = str(kw.pop('timestamp', None) or utc_now())

        self.nonce = kw.pop('nonce', None)
        if self.nonce is None:
            self.nonce = random_string(6)

        # This is a lookup function for checking nonces.
        self.seen_nonce = kw.pop('seen_nonce', None)

        self.url = kw.pop('url')
        if not self.url:
            raise ValueError('url was empty')
        url_parts = self.parse_url(self.url)
        log.debug('parsed URL parts: \n{parts}'
                  .format(parts=pprint.pformat(url_parts)))

        self.name = url_parts['resource'] or ''
        self.host = url_parts['hostname'] or ''
        self.port = str(url_parts['port'])

        if kw.keys():
            raise TypeError('Unknown keyword argument(s): {0}'
                            .format(kw.keys()))

    @property
    def content_hash(self):
        if not hasattr(self, '_content_hash'):
            raise AttributeError(
                'Cannot access content_hash because it has not been generated')
        return self._content_hash

    def gen_content_hash(self):
        if self.content is None or self.content_type is None:
            if self.always_hash_content:
                # Be really strict about allowing developers to skip content
                # hashing. If they get this far they may be unintentiionally
                # skipping it.
                raise ValueError(
                    'payload content and/or content_type cannot be '
                    'empty without an explicit allowance')
            log.debug('NOT hashing content')
            self._content_hash = None
        else:
            self._content_hash = calculate_payload_hash(
                self.content, self.credentials['algorithm'],
                self.content_type)
        return self.content_hash

    def parse_url(self, url):
        url_parts = urlparse(url)
        url_dict = {
            'scheme': url_parts.scheme,
            'hostname': url_parts.hostname,
            'port': url_parts.port,
            'path': url_parts.path,
            'resource': url_parts.path,
            'query': url_parts.query,
        }
        if len(url_dict['query']) > 0:
            url_dict['resource'] = '%s?%s' % (url_dict['resource'],
                                              url_dict['query'])

        if url_parts.port is None:
            if url_parts.scheme == 'http':
                url_dict['port'] = 80
            elif url_parts.scheme == 'https':
                url_dict['port'] = 443
        return url_dict
