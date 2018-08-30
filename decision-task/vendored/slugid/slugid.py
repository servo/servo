# Licensed under the Mozilla Public Licence 2.0.
# https://www.mozilla.org/en-US/MPL/2.0

import sys
import uuid
import base64


def encode(uuid_):
    """
    Returns the given uuid.UUID object as a 22 character slug. This can be a
    regular v4 slug or a "nice" slug.
    """
    return base64.urlsafe_b64encode(uuid_.bytes)[:-2]  # Drop '==' padding


def decode(slug):
    """
    Returns the uuid.UUID object represented by the given v4 or "nice" slug
    """
    if sys.version_info.major != 2 and isinstance(slug, bytes):
        slug = slug.decode('ascii')
    slug = slug + '=='  # base64 padding
    return uuid.UUID(bytes=base64.urlsafe_b64decode(slug))


def v4():
    """
    Returns a randomly generated uuid v4 compliant slug
    """
    return base64.urlsafe_b64encode(uuid.uuid4().bytes)[:-2]  # Drop '==' padding


def nice():
    """
    Returns a randomly generated uuid v4 compliant slug which conforms to a set
    of "nice" properties, at the cost of some entropy. Currently this means one
    extra fixed bit (the first bit of the uuid is set to 0) which guarantees the
    slug will begin with [A-Za-f]. For example such slugs don't require special
    handling when used as command line parameters (whereas non-nice slugs may
    start with `-` which can confuse command line tools).

    Potentially other "nice" properties may be added in future to further
    restrict the range of potential uuids that may be generated.
    """
    rawBytes = bytearray(uuid.uuid4().bytes)
    rawBytes[0] = rawBytes[0] & 0x7f  # Ensure slug starts with [A-Za-f]
    return base64.urlsafe_b64encode(rawBytes)[:-2]  # Drop '==' padding
