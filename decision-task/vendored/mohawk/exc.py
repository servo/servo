"""
If you want to catch any exception that might be raised,
catch :class:`mohawk.exc.HawkFail`.
"""


class HawkFail(Exception):
    """
    All Mohawk exceptions derive from this base.
    """


class MissingAuthorization(HawkFail):
    """
    No authorization header was sent by the client.
    """


class InvalidCredentials(HawkFail):
    """
    The specified Hawk credentials are invalid.

    For example, the dict could be formatted incorrectly.
    """


class CredentialsLookupError(HawkFail):
    """
    A :class:`mohawk.Receiver` could not look up the
    credentials for an incoming request.
    """


class BadHeaderValue(HawkFail):
    """
    There was an error with an attribute or value when parsing
    or creating a Hawk header.
    """


class MacMismatch(HawkFail):
    """
    The locally calculated MAC did not match the MAC that was sent.
    """


class MisComputedContentHash(HawkFail):
    """
    The signature of the content did not match the actual content.
    """


class TokenExpired(HawkFail):
    """
    The timestamp on a message received has expired.

    You may also receive this message if your server clock is out of sync.
    Consider synchronizing it with something like `TLSdate`_.

    If you are unable to synchronize your clock universally,
    The `Hawk`_ spec mentions how you can `adjust`_
    your sender's time to match that of the receiver in the case
    of unexpected expiration.

    The ``www_authenticate`` attribute of this exception is a header
    that can be returned to the client. If the value is not None, it
    will include a timestamp HMAC'd with the sender's credentials.
    This will allow the client
    to verify the value and safely apply an offset.

    .. _`Hawk`: https://github.com/hueniverse/hawk
    .. _`adjust`: https://github.com/hueniverse/hawk#future-time-manipulation
    .. _`TLSdate`: http://linux-audit.com/tlsdate-the-secure-alternative-for-ntpd-ntpdate-and-rdate/
    """
    #: Current local time in seconds that was used to compare timestamps.
    localtime_in_seconds = None
    # A header containing an HMAC'd server timestamp that the sender can verify.
    www_authenticate = None

    def __init__(self, *args, **kw):
        self.localtime_in_seconds = kw.pop('localtime_in_seconds')
        self.www_authenticate = kw.pop('www_authenticate')
        super(HawkFail, self).__init__(*args, **kw)


class AlreadyProcessed(HawkFail):
    """
    The message has already been processed and cannot be re-processed.

    See :ref:`nonce` for details.
    """


class InvalidBewit(HawkFail):
    """
    The bewit is invalid; e.g. it doesn't contain the right number of
    parameters.
    """
