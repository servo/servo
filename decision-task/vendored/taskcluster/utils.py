from __future__ import absolute_import, division, print_function
import re
import json
import datetime
import base64
import logging
import os
import requests
import requests.exceptions
import slugid
import time
import six
import sys
import random

from . import exceptions

MAX_RETRIES = 5

DELAY_FACTOR = 0.1
RANDOMIZATION_FACTOR = 0.25
MAX_DELAY = 30


log = logging.getLogger(__name__)

# Regular expression matching: X days Y hours Z minutes
# todo: support hr, wk, yr
r = re.compile(''.join([
   '^(\s*(?P<years>\d+)\s*y(ears?)?)?',
   '(\s*(?P<months>\d+)\s*mo(nths?)?)?',
   '(\s*(?P<weeks>\d+)\s*w(eeks?)?)?',
   '(\s*(?P<days>\d+)\s*d(ays?)?)?',
   '(\s*(?P<hours>\d+)\s*h(ours?)?)?',
   '(\s*(?P<minutes>\d+)\s*m(in(utes?)?)?)?\s*',
   '(\s*(?P<seconds>\d+)\s*s(ec(onds?)?)?)?\s*$',
]))


def calculateSleepTime(attempt):
    """ From the go client
    https://github.com/taskcluster/go-got/blob/031f55c/backoff.go#L24-L29
    """
    if attempt <= 0:
        return 0

    # We subtract one to get exponents: 1, 2, 3, 4, 5, ..
    delay = float(2 ** (attempt - 1)) * float(DELAY_FACTOR)
    # Apply randomization factor
    delay = delay * (RANDOMIZATION_FACTOR * (random.random() * 2 - 1) + 1)
    # Always limit with a maximum delay
    return min(delay, MAX_DELAY)


def toStr(obj, encoding='utf-8'):
    if six.PY3 and isinstance(obj, six.binary_type):
        obj = obj.decode(encoding)
    else:
        obj = str(obj)
    return obj


def fromNow(offset, dateObj=None):
    """
    Generate a `datetime.datetime` instance which is offset using a string.
    See the README.md for a full example, but offset could be '1 day' for
    a datetime object one day in the future
    """

    # We want to handle past dates as well as future
    future = True
    offset = offset.lstrip()
    if offset.startswith('-'):
        future = False
        offset = offset[1:].lstrip()
    if offset.startswith('+'):
        offset = offset[1:].lstrip()

    # Parse offset
    m = r.match(offset)
    if m is None:
        raise ValueError("offset string: '%s' does not parse" % offset)

    # In order to calculate years and months we need to calculate how many days
    # to offset the offset by, since timedelta only goes as high as weeks
    days = 0
    hours = 0
    minutes = 0
    seconds = 0
    if m.group('years'):
        years = int(m.group('years'))
        days += 365 * years
    if m.group('months'):
        months = int(m.group('months'))
        days += 30 * months
    days += int(m.group('days') or 0)
    hours += int(m.group('hours') or 0)
    minutes += int(m.group('minutes') or 0)
    seconds += int(m.group('seconds') or 0)

    # Offset datetime from utc
    delta = datetime.timedelta(
        weeks=int(m.group('weeks') or 0),
        days=days,
        hours=hours,
        minutes=minutes,
        seconds=seconds,
    )

    if not dateObj:
        dateObj = datetime.datetime.utcnow()

    return dateObj + delta if future else dateObj - delta


def fromNowJSON(offset):
    """
    Like fromNow() but returns in a taskcluster-json compatible way
    """
    return stringDate(fromNow(offset))


def dumpJson(obj, **kwargs):
    """ Match JS's JSON.stringify.  When using the default seperators,
    base64 encoding JSON results in \n sequences in the output.  Hawk
    barfs in your face if you have that in the text"""
    def handleDateAndBinaryForJs(x):
        if six.PY3 and isinstance(x, six.binary_type):
            x = x.decode()
        if isinstance(x, datetime.datetime) or isinstance(x, datetime.date):
            return stringDate(x)
        else:
            return x
    d = json.dumps(obj, separators=(',', ':'), default=handleDateAndBinaryForJs, **kwargs)
    assert '\n' not in d
    return d


def stringDate(date):
    # Convert to isoFormat
    string = date.isoformat()

    # If there is no timezone and no Z added, we'll add one at the end.
    # This is just to be fully compliant with:
    # https://tools.ietf.org/html/rfc3339#section-5.6
    if string.endswith('+00:00'):
        return string[:-6] + 'Z'
    if date.utcoffset() is None and string[-1] != 'Z':
        return string + 'Z'
    return string


def makeB64UrlSafe(b64str):
    """ Make a base64 string URL Safe """
    if isinstance(b64str, six.text_type):
        b64str = b64str.encode()
    # see RFC 4648, sec. 5
    return b64str.replace(b'+', b'-').replace(b'/', b'_')


def makeB64UrlUnsafe(b64str):
    """ Make a base64 string URL Unsafe """
    if isinstance(b64str, six.text_type):
        b64str = b64str.encode()
    # see RFC 4648, sec. 5
    return b64str.replace(b'-', b'+').replace(b'_', b'/')


def encodeStringForB64Header(s):
    """ HTTP Headers can't have new lines in them, let's """
    if isinstance(s, six.text_type):
        s = s.encode()
    return base64.encodestring(s).strip().replace(b'\n', b'')


def slugId():
    """ Generate a taskcluster slugid.  This is a V4 UUID encoded into
    URL-Safe Base64 (RFC 4648, sec 5) with '=' padding removed """
    return slugid.nice()


def stableSlugId():
    """Returns a closure which can be used to generate stable slugIds.
    Stable slugIds can be used in a graph to specify task IDs in multiple
    places without regenerating them, e.g. taskId, requires, etc.
    """
    _cache = {}

    def closure(name):
        if name not in _cache:
            _cache[name] = slugId()
        return _cache[name]

    return closure


def scopeMatch(assumedScopes, requiredScopeSets):
    """
        Take a list of a assumed scopes, and a list of required scope sets on
        disjunctive normal form, and check if any of the required scope sets are
        satisfied.

        Example:

            requiredScopeSets = [
                ["scopeA", "scopeB"],
                ["scopeC"]
            ]

        In this case assumed_scopes must contain, either:
        "scopeA" AND "scopeB", OR just "scopeC".
    """
    for scopeSet in requiredScopeSets:
        for requiredScope in scopeSet:
            for scope in assumedScopes:
                if scope == requiredScope:
                    # requiredScope satisifed, no need to check more scopes
                    break
                if scope.endswith("*") and requiredScope.startswith(scope[:-1]):
                    # requiredScope satisifed, no need to check more scopes
                    break
            else:
                # requiredScope not satisfied, stop checking scopeSet
                break
        else:
            # scopeSet satisfied, so we're happy
            return True
    # none of the requiredScopeSets were satisfied
    return False


def scope_match(assumed_scopes, required_scope_sets):
    """ This is a deprecated form of def scopeMatch(assumedScopes, requiredScopeSets).
    That form should be used.
    """
    import warnings
    warnings.warn('NOTE: scope_match is deprecated.  Use scopeMatch')
    return scopeMatch(assumed_scopes, required_scope_sets)


def makeHttpRequest(method, url, payload, headers, retries=MAX_RETRIES, session=None):
    """ Make an HTTP request and retry it until success, return request """
    retry = -1
    response = None
    while retry < retries:
        retry += 1
        # if this isn't the first retry then we sleep
        if retry > 0:
            snooze = float(retry * retry) / 10.0
            log.info('Sleeping %0.2f seconds for exponential backoff', snooze)
            time.sleep(snooze)

        # Seek payload to start, if it is a file
        if hasattr(payload, 'seek'):
            payload.seek(0)

        log.debug('Making attempt %d', retry)
        try:
            response = makeSingleHttpRequest(method, url, payload, headers, session)
        except requests.exceptions.RequestException as rerr:
            if retry < retries:
                log.warn('Retrying because of: %s' % rerr)
                continue
            # raise a connection exception
            raise rerr
        # Handle non 2xx status code and retry if possible
        try:
            response.raise_for_status()
        except requests.exceptions.RequestException as rerr:
            pass
        status = response.status_code
        if 500 <= status and status < 600 and retry < retries:
            if retry < retries:
                log.warn('Retrying because of: %d status' % status)
                continue
            else:
                raise exceptions.TaskclusterRestFailure("Unknown Server Error", superExc=None)
        return response

    # This code-path should be unreachable
    assert False, "Error from last retry should have been raised!"


def makeSingleHttpRequest(method, url, payload, headers, session=None):
    method = method.upper()
    log.debug('Making a %s request to %s', method, url)
    log.debug('HTTP Headers: %s' % str(headers))
    log.debug('HTTP Payload: %s (limit 100 char)' % str(payload)[:100])
    obj = session if session else requests
    response = obj.request(method.upper(), url, data=payload, headers=headers)
    log.debug('Received HTTP Status:    %s' % response.status_code)
    log.debug('Received HTTP Headers: %s' % str(response.headers))

    return response


def putFile(filename, url, contentType):
    with open(filename, 'rb') as f:
        contentLength = os.fstat(f.fileno()).st_size
        return makeHttpRequest('put', url, f, headers={
            'Content-Length': contentLength,
            'Content-Type': contentType,
        })


def encryptEnvVar(taskId, startTime, endTime, name, value, keyFile):
    raise Exception("Encrypted environment variables are no longer supported")


def decryptMessage(message, privateKey):
    raise Exception("Decryption is no longer supported")


def isExpired(certificate):
    """ Check if certificate is expired """
    if isinstance(certificate, six.string_types):
        certificate = json.loads(certificate)
    expiry = certificate.get('expiry', 0)
    return expiry < int(time.time() * 1000) + 20 * 60


def authenticate(description=None):
    """
    Open a web-browser to login.taskcluster.net and listen on localhost for
    a callback with credentials in query-string.

    The description will be shown on login.taskcluster.net, if not provided
    a default message with script path will be displayed.
    """
    # Importing here to avoid loading these 'obscure' module before it's needed.
    # Most clients won't use this feature, so we don't want issues with these
    # modules to affect the library. Maybe they don't work in some environments
    import webbrowser
    from six.moves import urllib
    from six.moves.urllib.parse import quote
    import BaseHTTPServer

    if not description:
        script = '[interpreter/unknown]'
        main = sys.modules.get('__main__', None)
        if main and hasattr(main, '__file__'):
            script = os.path.abspath(main.__file__)
        description = (
            "Python script: `%s`\n\nWould like some temporary credentials."
            % script
        )

    creds = [None]

    class AuthCallBackRequestHandler(BaseHTTPServer.BaseHTTPRequestHandler):
        def log_message(format, *args):
            pass

        def do_GET(self):
            url = urllib.parse.urlparse(self.path)
            query = urllib.parse.parse_qs(url.query)
            clientId = query.get('clientId', [None])[0]
            accessToken = query.get('accessToken', [None])[0]
            certificate = query.get('certificate', [None])[0]
            hasCreds = clientId and accessToken and certificate
            if hasCreds:
                creds[0] = {
                    "clientId": clientId,
                    "accessToken": accessToken,
                    "certificate": certificate
                }
            self.send_response(200)
            self.send_header('Content-type', 'text/html')
            self.end_headers()
            if hasCreds:
                self.wfile.write("""
                    <h1>Credentials transferred successfully</h1>
                    <i>You can close this window now.</i>
                    <script>window.close();</script>
                """)
            else:
                self.wfile.write("""
                    <h1>Transfer of credentials failed!</h1>
                    <p>Something went wrong, you can navigate back and try again...</p>
                """)
            return

    # Create server on localhost at random port
    retries = 5
    while retries > 0:
        try:
            server = BaseHTTPServer.HTTPServer(('', 0), AuthCallBackRequestHandler)
        except:
            retries -= 1
        break
    port = server.server_address[1]

    query = "?target=" + quote('http://localhost:' + str(port), '')
    query += "&description=" + quote(description, '')

    webbrowser.open('https://login.taskcluster.net' + query, 1, True)
    print("")
    print("-------------------------------------------------------")
    print("  Opening browser window to login.taskcluster.net")
    print("  Asking you to grant temporary credentials to:")
    print("     http://localhost:" + str(port))
    print("-------------------------------------------------------")
    print("")

    while not creds[0]:
        server.handle_request()
    return creds[0]
