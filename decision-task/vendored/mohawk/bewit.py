from base64 import urlsafe_b64encode, b64decode
from collections import namedtuple
import logging
import re

import six

from .base import Resource
from .util import (calculate_mac,
                   utc_now)
from .exc import (CredentialsLookupError,
                  InvalidBewit,
                  MacMismatch,
                  TokenExpired)

log = logging.getLogger(__name__)


def get_bewit(resource):
    """
    Returns a bewit identifier for the resource as a string.

    :param resource:
        Resource to generate a bewit for
    :type resource: `mohawk.base.Resource`
    """
    if resource.method != 'GET':
        raise ValueError('bewits can only be generated for GET requests')
    if resource.nonce != '':
        raise ValueError('bewits must use an empty nonce')
    mac = calculate_mac(
        'bewit',
        resource,
        None,
    )

    if isinstance(mac, six.binary_type):
        mac = mac.decode('ascii')

    if resource.ext is None:
        ext = ''
    else:
        ext = resource.ext

    # Strip out \ from the client id
    # since that can break parsing the response
    # NB that the canonical implementation does not do this as of
    # Oct 28, 2015, so this could break compat.
    # We can leave \ in ext since validators can limit how many \ they split
    # on (although again, the canonical implementation does not do this)
    client_id = six.text_type(resource.credentials['id'])
    if "\\" in client_id:
        log.warn("Stripping backslash character(s) '\\' from client_id")
        client_id = client_id.replace("\\", "")

    # b64encode works only with bytes in python3, but all of our parameters are
    # in unicode, so we need to encode them. The cleanest way to do this that
    # works in both python 2 and 3 is to use string formatting to get a
    # unicode string, and then explicitly encode it to bytes.
    inner_bewit = u"{id}\\{exp}\\{mac}\\{ext}".format(
        id=client_id,
        exp=resource.timestamp,
        mac=mac,
        ext=ext,
    )
    inner_bewit_bytes = inner_bewit.encode('ascii')
    bewit_bytes = urlsafe_b64encode(inner_bewit_bytes)
    # Now decode the resulting bytes back to a unicode string
    return bewit_bytes.decode('ascii')


bewittuple = namedtuple('bewittuple', 'id expiration mac ext')


def parse_bewit(bewit):
    """
    Returns a `bewittuple` representing the parts of an encoded bewit string.
    This has the following named attributes:
        (id, expiration, mac, ext)

    :param bewit:
        A base64 encoded bewit string
    :type bewit: str
    """
    decoded_bewit = b64decode(bewit).decode('ascii')
    bewit_parts = decoded_bewit.split("\\", 3)
    if len(bewit_parts) != 4:
        raise InvalidBewit('Expected 4 parts to bewit: %s' % decoded_bewit)
    return bewittuple(*decoded_bewit.split("\\", 3))


def strip_bewit(url):
    """
    Strips the bewit parameter out of a url.

    Returns (encoded_bewit, stripped_url)

    Raises InvalidBewit if no bewit found.

    :param url:
        The url containing a bewit parameter
    :type url: str
    """
    m = re.search('[?&]bewit=([^&]+)', url)
    if not m:
        raise InvalidBewit('no bewit data found')
    bewit = m.group(1)
    stripped_url = url[:m.start()] + url[m.end():]
    return bewit, stripped_url


def check_bewit(url, credential_lookup, now=None):
    """
    Validates the given bewit.

    Returns True if the resource has a valid bewit parameter attached,
    or raises a subclass of HawkFail otherwise.

    :param credential_lookup:
        Callable to look up the credentials dict by sender ID.
        The credentials dict must have the keys:
        ``id``, ``key``, and ``algorithm``.
        See :ref:`receiving-request` for an example.
    :type credential_lookup: callable

    :param now=None:
        Unix epoch time for the current time to determine if bewit has expired.
        If None, then the current time as given by utc_now() is used.
    :type now=None: integer
    """
    raw_bewit, stripped_url = strip_bewit(url)
    bewit = parse_bewit(raw_bewit)
    try:
        credentials = credential_lookup(bewit.id)
    except LookupError:
        raise CredentialsLookupError('Could not find credentials for ID {0}'
                                     .format(bewit.id))

    res = Resource(url=stripped_url,
                   method='GET',
                   credentials=credentials,
                   timestamp=bewit.expiration,
                   nonce='',
                   ext=bewit.ext,
                   )
    mac = calculate_mac('bewit', res, None)
    mac = mac.decode('ascii')

    if mac != bewit.mac:
        raise MacMismatch('bewit with mac {bewit_mac} did not match expected mac {expected_mac}'
                          .format(bewit_mac=bewit.mac,
                                  expected_mac=mac))

    # Check that the timestamp isn't expired
    if now is None:
        # TODO: Add offset/skew
        now = utc_now()
    if int(bewit.expiration) < now:
        # TODO: Refactor TokenExpired to handle this better
        raise TokenExpired('bewit with UTC timestamp {ts} has expired; '
                           'it was compared to {now}'
                           .format(ts=bewit.expiration, now=now),
                           localtime_in_seconds=now,
                           www_authenticate=''
                           )

    return True
