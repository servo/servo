"""This module is used to interact with taskcluster rest apis"""

from __future__ import absolute_import, division, print_function

import os
import logging
import hashlib
import hmac
import datetime
import calendar
import six
from six.moves import urllib

import mohawk
import mohawk.bewit
import aiohttp
import asyncio

from .. import exceptions
from .. import utils
from ..client import BaseClient
from . import asyncutils

log = logging.getLogger(__name__)


# Default configuration
_defaultConfig = config = {
    'credentials': {
        'clientId': os.environ.get('TASKCLUSTER_CLIENT_ID'),
        'accessToken': os.environ.get('TASKCLUSTER_ACCESS_TOKEN'),
        'certificate': os.environ.get('TASKCLUSTER_CERTIFICATE'),
    },
    'maxRetries': 5,
    'signedUrlExpiration': 15 * 60,
}


def createSession(*args, **kwargs):
    """ Create a new aiohttp session.  This passes through all positional and
    keyword arguments to the asyncutils.createSession() constructor.

    It's preferred to do something like

        async with createSession(...) as session:
            queue = Queue(session=session)
            await queue.ping()

    or

        async with createSession(...) as session:
            async with Queue(session=session) as queue:
                await queue.ping()

    in the client code.
    """
    return asyncutils.createSession(*args, **kwargs)


class AsyncBaseClient(BaseClient):
    """ Base Class for API Client Classes. Each individual Client class
    needs to set up its own methods for REST endpoints and Topic Exchange
    routing key patterns.  The _makeApiCall() and _topicExchange() methods
    help with this.
    """

    def __init__(self, *args, **kwargs):
        super(AsyncBaseClient, self).__init__(*args, **kwargs)
        self._implicitSession = False
        if self.session is None:
            self._implicitSession = True

    def _createSession(self):
        """ If self.session isn't set, don't create an implicit.

        To avoid `session.close()` warnings at the end of tasks, and
        various strongly-worded aiohttp warnings about using `async with`,
        let's set `self.session` to `None` if no session is passed in to
        `__init__`. The `asyncutils` functions will create a new session
        per call in that case.
        """
        return None

    async def _makeApiCall(self, entry, *args, **kwargs):
        """ This function is used to dispatch calls to other functions
        for a given API Reference entry"""

        x = self._processArgs(entry, *args, **kwargs)
        routeParams, payload, query, paginationHandler, paginationLimit = x
        route = self._subArgsInRoute(entry, routeParams)

        # TODO: Check for limit being in the Query of the api ref
        if paginationLimit and 'limit' in entry.get('query', []):
            query['limit'] = paginationLimit

        if query:
            _route = route + '?' + urllib.parse.urlencode(query)
        else:
            _route = route
        response = await self._makeHttpRequest(entry['method'], _route, payload)

        if paginationHandler:
            paginationHandler(response)
            while response.get('continuationToken'):
                query['continuationToken'] = response['continuationToken']
                _route = route + '?' + urllib.parse.urlencode(query)
                response = await self._makeHttpRequest(entry['method'], _route, payload)
                paginationHandler(response)
        else:
            return response

    async def _makeHttpRequest(self, method, route, payload):
        """ Make an HTTP Request for the API endpoint.  This method wraps
        the logic about doing failure retry and passes off the actual work
        of doing an HTTP request to another method."""

        url = self._joinBaseUrlAndRoute(route)
        log.debug('Full URL used is: %s', url)

        hawkExt = self.makeHawkExt()

        # Serialize payload if given
        if payload is not None:
            payload = utils.dumpJson(payload)

        # Do a loop of retries
        retry = -1  # we plus first in the loop, and attempt 1 is retry 0
        retries = self.options['maxRetries']
        while retry < retries:
            retry += 1
            # if this isn't the first retry then we sleep
            if retry > 0:
                snooze = float(retry * retry) / 10.0
                log.info('Sleeping %0.2f seconds for exponential backoff', snooze)
                await asyncio.sleep(utils.calculateSleepTime(retry))
            # Construct header
            if self._hasCredentials():
                sender = mohawk.Sender(
                    credentials={
                        'id': self.options['credentials']['clientId'],
                        'key': self.options['credentials']['accessToken'],
                        'algorithm': 'sha256',
                    },
                    ext=hawkExt if hawkExt else {},
                    url=url,
                    content=payload if payload else '',
                    content_type='application/json' if payload else '',
                    method=method,
                )

                headers = {'Authorization': sender.request_header}
            else:
                log.debug('Not using hawk!')
                headers = {}
            if payload:
                # Set header for JSON if payload is given, note that we serialize
                # outside this loop.
                headers['Content-Type'] = 'application/json'

            log.debug('Making attempt %d', retry)
            try:
                response = await asyncutils.makeSingleHttpRequest(
                    method, url, payload, headers, session=self.session
                )
            except aiohttp.ClientError as rerr:
                if retry < retries:
                    log.warn('Retrying because of: %s' % rerr)
                    continue
                # raise a connection exception
                raise exceptions.TaskclusterConnectionError(
                    "Failed to establish connection",
                    superExc=rerr
                )

            status = response.status
            if status == 204:
                return None

            # Catch retryable errors and go to the beginning of the loop
            # to do the retry
            if 500 <= status and status < 600 and retry < retries:
                log.warn('Retrying because of a %s status code' % status)
                continue

            # Throw errors for non-retryable errors
            if status < 200 or status >= 300:
                # Parse messages from errors
                data = {}
                try:
                    data = await response.json()
                except:
                    pass  # Ignore JSON errors in error messages
                # Find error message
                message = "Unknown Server Error"
                if isinstance(data, dict):
                    message = data.get('message')
                else:
                    if status == 401:
                        message = "Authentication Error"
                    elif status == 500:
                        message = "Internal Server Error"
                    else:
                        message = "Unknown Server Error %s\n%s" % (str(status), str(data)[:1024])
                # Raise TaskclusterAuthFailure if this is an auth issue
                if status == 401:
                    raise exceptions.TaskclusterAuthFailure(
                        message,
                        status_code=status,
                        body=data,
                        superExc=None
                    )
                # Raise TaskclusterRestFailure for all other issues
                raise exceptions.TaskclusterRestFailure(
                    message,
                    status_code=status,
                    body=data,
                    superExc=None
                )

            # Try to load JSON
            try:
                await response.release()
                return await response.json()
            except ValueError:
                return {"response": response}

        # This code-path should be unreachable
        assert False, "Error from last retry should have been raised!"

    async def __aenter__(self):
        if self._implicitSession and not self.session:
            self.session = createSession()
        return self

    async def __aexit__(self, *args):
        if self._implicitSession and self.session:
            await self.session.close()
            self.session = None


def createApiClient(name, api):
    attributes = dict(
        name=name,
        __doc__=api.get('description'),
        classOptions={},
        funcinfo={},
    )

    copiedOptions = ('baseUrl', 'exchangePrefix')
    for opt in copiedOptions:
        if opt in api['reference']:
            attributes['classOptions'][opt] = api['reference'][opt]

    for entry in api['reference']['entries']:
        if entry['type'] == 'function':
            def addApiCall(e):
                async def apiCall(self, *args, **kwargs):
                    return await self._makeApiCall(e, *args, **kwargs)
                return apiCall
            f = addApiCall(entry)

            docStr = "Call the %s api's %s method.  " % (name, entry['name'])

            if entry['args'] and len(entry['args']) > 0:
                docStr += "This method takes:\n\n"
                docStr += '\n'.join(['- ``%s``' % x for x in entry['args']])
                docStr += '\n\n'
            else:
                docStr += "This method takes no arguments.  "

            if 'input' in entry:
                docStr += "This method takes input ``%s``.  " % entry['input']

            if 'output' in entry:
                docStr += "This method gives output ``%s``" % entry['output']

            docStr += '\n\nThis method does a ``%s`` to ``%s``.' % (
                entry['method'].upper(), entry['route'])

            f.__doc__ = docStr
            attributes['funcinfo'][entry['name']] = entry

        elif entry['type'] == 'topic-exchange':
            def addTopicExchange(e):
                def topicExchange(self, *args, **kwargs):
                    return self._makeTopicExchange(e, *args, **kwargs)
                return topicExchange

            f = addTopicExchange(entry)

            docStr = 'Generate a routing key pattern for the %s exchange.  ' % entry['exchange']
            docStr += 'This method takes a given routing key as a string or a '
            docStr += 'dictionary.  For each given dictionary key, the corresponding '
            docStr += 'routing key token takes its value.  For routing key tokens '
            docStr += 'which are not specified by the dictionary, the * or # character '
            docStr += 'is used depending on whether or not the key allows multiple words.\n\n'
            docStr += 'This exchange takes the following keys:\n\n'
            docStr += '\n'.join(['- ``%s``' % x['name'] for x in entry['routingKey']])

            f.__doc__ = docStr

        # Add whichever function we created
        f.__name__ = str(entry['name'])
        attributes[entry['name']] = f

    return type(utils.toStr(name), (BaseClient,), attributes)


def createTemporaryCredentials(clientId, accessToken, start, expiry, scopes, name=None):
    """ Create a set of temporary credentials

    Callers should not apply any clock skew; clock drift is accounted for by
    auth service.

    clientId: the issuing clientId
    accessToken: the issuer's accessToken
    start: start time of credentials, seconds since epoch
    expiry: expiration time of credentials, seconds since epoch
    scopes: list of scopes granted
    name: credential name (optional)

    Returns a dictionary in the form:
        { 'clientId': str, 'accessToken: str, 'certificate': str}
    """

    now = datetime.datetime.utcnow()
    now = now - datetime.timedelta(minutes=10)  # Subtract 5 minutes for clock drift

    for scope in scopes:
        if not isinstance(scope, six.string_types):
            raise exceptions.TaskclusterFailure('Scope must be string')

    # Credentials can only be valid for 31 days.  I hope that
    # this is validated on the server somehow...

    if expiry - start > datetime.timedelta(days=31):
        raise exceptions.TaskclusterFailure('Only 31 days allowed')

    # We multiply times by 1000 because the auth service is JS and as a result
    # uses milliseconds instead of seconds
    cert = dict(
        version=1,
        scopes=scopes,
        start=calendar.timegm(start.utctimetuple()) * 1000,
        expiry=calendar.timegm(expiry.utctimetuple()) * 1000,
        seed=utils.slugId() + utils.slugId(),
    )

    # if this is a named temporary credential, include the issuer in the certificate
    if name:
        cert['issuer'] = utils.toStr(clientId)

    sig = ['version:' + utils.toStr(cert['version'])]
    if name:
        sig.extend([
            'clientId:' + utils.toStr(name),
            'issuer:' + utils.toStr(clientId),
        ])
    sig.extend([
        'seed:' + utils.toStr(cert['seed']),
        'start:' + utils.toStr(cert['start']),
        'expiry:' + utils.toStr(cert['expiry']),
        'scopes:'
    ] + scopes)
    sigStr = '\n'.join(sig).encode()

    if isinstance(accessToken, six.text_type):
        accessToken = accessToken.encode()
    sig = hmac.new(accessToken, sigStr, hashlib.sha256).digest()

    cert['signature'] = utils.encodeStringForB64Header(sig)

    newToken = hmac.new(accessToken, cert['seed'], hashlib.sha256).digest()
    newToken = utils.makeB64UrlSafe(utils.encodeStringForB64Header(newToken)).replace(b'=', b'')

    return {
        'clientId': name or clientId,
        'accessToken': newToken,
        'certificate': utils.dumpJson(cert),
    }


__all__ = [
    'createTemporaryCredentials',
    'config',
    'BaseClient',
    'createApiClient',
]
