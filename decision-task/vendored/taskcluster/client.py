"""This module is used to interact with taskcluster rest apis"""

from __future__ import absolute_import, division, print_function

import os
import json
import logging
import copy
import hashlib
import hmac
import datetime
import calendar
import requests
import time
import six
import warnings
from six.moves import urllib

import mohawk
import mohawk.bewit

import taskcluster.exceptions as exceptions
import taskcluster.utils as utils

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
    """ Create a new requests session.  This passes through all positional and
    keyword arguments to the requests.Session() constructor
    """
    return requests.Session(*args, **kwargs)


class BaseClient(object):
    """ Base Class for API Client Classes. Each individual Client class
    needs to set up its own methods for REST endpoints and Topic Exchange
    routing key patterns.  The _makeApiCall() and _topicExchange() methods
    help with this.
    """

    def __init__(self, options=None, session=None):
        o = copy.deepcopy(self.classOptions)
        o.update(_defaultConfig)
        if options:
            o.update(options)

        credentials = o.get('credentials')
        if credentials:
            for x in ('accessToken', 'clientId', 'certificate'):
                value = credentials.get(x)
                if value and not isinstance(value, six.binary_type):
                    try:
                        credentials[x] = credentials[x].encode('ascii')
                    except:
                        s = '%s (%s) must be unicode encodable' % (x, credentials[x])
                        raise exceptions.TaskclusterAuthFailure(s)
        self.options = o
        if 'credentials' in o:
            log.debug('credentials key scrubbed from logging output')
        log.debug(dict((k, v) for k, v in o.items() if k != 'credentials'))

        if session:
            self.session = session
        else:
            self.session = self._createSession()

    def _createSession(self):
        """ Create a requests session.

        Helper method which can be overridden by child classes.
        """
        return createSession()

    def makeHawkExt(self):
        """ Make an 'ext' for Hawk authentication """
        o = self.options
        c = o.get('credentials', {})
        if c.get('clientId') and c.get('accessToken'):
            ext = {}
            cert = c.get('certificate')
            if cert:
                if six.PY3 and isinstance(cert, six.binary_type):
                    cert = cert.decode()
                if isinstance(cert, six.string_types):
                    cert = json.loads(cert)
                ext['certificate'] = cert

            if 'authorizedScopes' in o:
                ext['authorizedScopes'] = o['authorizedScopes']

            # .encode('base64') inserts a newline, which hawk doesn't
            # like but doesn't strip itself
            return utils.makeB64UrlSafe(utils.encodeStringForB64Header(utils.dumpJson(ext)).strip())
        else:
            return {}

    def _makeTopicExchange(self, entry, *args, **kwargs):
        if len(args) == 0 and not kwargs:
            routingKeyPattern = {}
        elif len(args) >= 1:
            if kwargs or len(args) != 1:
                errStr = 'Pass either a string, single dictionary or only kwargs'
                raise exceptions.TaskclusterTopicExchangeFailure(errStr)
            routingKeyPattern = args[0]
        else:
            routingKeyPattern = kwargs

        data = {
            'exchange': '%s/%s' % (self.options['exchangePrefix'].rstrip('/'),
                                   entry['exchange'].lstrip('/'))
        }

        # If we are passed in a string, we can short-circuit this function
        if isinstance(routingKeyPattern, six.string_types):
            log.debug('Passing through string for topic exchange key')
            data['routingKeyPattern'] = routingKeyPattern
            return data

        if type(routingKeyPattern) != dict:
            errStr = 'routingKeyPattern must eventually be a dict'
            raise exceptions.TaskclusterTopicExchangeFailure(errStr)

        if not routingKeyPattern:
            routingKeyPattern = {}

        # There is no canonical meaning for the maxSize and required
        # reference entry in the JS client, so we don't try to define
        # them here, even though they sound pretty obvious

        routingKey = []
        for key in entry['routingKey']:
            if 'constant' in key:
                value = key['constant']
            elif key['name'] in routingKeyPattern:
                log.debug('Found %s in routing key params', key['name'])
                value = str(routingKeyPattern[key['name']])
                if not key.get('multipleWords') and '.' in value:
                    raise exceptions.TaskclusterTopicExchangeFailure(
                        'Cannot have periods in single word keys')
            else:
                value = '#' if key.get('multipleWords') else '*'
                log.debug('Did not find %s in input params, using %s', key['name'], value)

            routingKey.append(value)

        data['routingKeyPattern'] = '.'.join([str(x) for x in routingKey])
        return data

    def buildUrl(self, methodName, *args, **kwargs):
        entry = self.funcinfo.get(methodName)
        if not entry:
            raise exceptions.TaskclusterFailure(
                'Requested method "%s" not found in API Reference' % methodName)
        routeParams, _, query, _, _ = self._processArgs(entry, *args, **kwargs)
        route = self._subArgsInRoute(entry, routeParams)
        if query:
            route += '?' + urllib.parse.urlencode(query)
        return self._joinBaseUrlAndRoute(route)

    def buildSignedUrl(self, methodName, *args, **kwargs):
        """ Build a signed URL.  This URL contains the credentials needed to access
        a resource."""

        if 'expiration' in kwargs:
            expiration = kwargs['expiration']
            del kwargs['expiration']
        else:
            expiration = self.options['signedUrlExpiration']

        expiration = int(time.time() + expiration)  # Mainly so that we throw if it's not a number

        requestUrl = self.buildUrl(methodName, *args, **kwargs)

        if not self._hasCredentials():
            raise exceptions.TaskclusterAuthFailure('Invalid Hawk Credentials')

        clientId = utils.toStr(self.options['credentials']['clientId'])
        accessToken = utils.toStr(self.options['credentials']['accessToken'])

        def genBewit():
            # We need to fix the output of get_bewit.  It returns a url-safe base64
            # encoded string, which contains a list of tokens separated by '\'.
            # The first one is the clientId, the second is an int, the third is
            # url-safe base64 encoded MAC, the fourth is the ext param.
            # The problem is that the nested url-safe base64 encoded MAC must be
            # base64 (i.e. not url safe) or server-side will complain.

            # id + '\\' + exp + '\\' + mac + '\\' + options.ext;
            resource = mohawk.base.Resource(
                credentials={
                    'id': clientId,
                    'key': accessToken,
                    'algorithm': 'sha256',
                },
                method='GET',
                ext=utils.toStr(self.makeHawkExt()),
                url=requestUrl,
                timestamp=expiration,
                nonce='',
                # content='',
                # content_type='',
            )
            bewit = mohawk.bewit.get_bewit(resource)
            return bewit.rstrip('=')

        bewit = genBewit()

        if not bewit:
            raise exceptions.TaskclusterFailure('Did not receive a bewit')

        u = urllib.parse.urlparse(requestUrl)

        qs = u.query
        if qs:
            qs += '&'
        qs += 'bewit=%s' % bewit

        return urllib.parse.urlunparse((
            u.scheme,
            u.netloc,
            u.path,
            u.params,
            qs,
            u.fragment,
        ))

    def _joinBaseUrlAndRoute(self, route):
        return urllib.parse.urljoin(
            '{}/'.format(self.options['baseUrl'].rstrip('/')),
            route.lstrip('/')
        )

    def _makeApiCall(self, entry, *args, **kwargs):
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
        response = self._makeHttpRequest(entry['method'], _route, payload)

        if paginationHandler:
            paginationHandler(response)
            while response.get('continuationToken'):
                query['continuationToken'] = response['continuationToken']
                _route = route + '?' + urllib.parse.urlencode(query)
                response = self._makeHttpRequest(entry['method'], _route, payload)
                paginationHandler(response)
        else:
            return response

    def _processArgs(self, entry, *_args, **_kwargs):
        """ Given an entry, positional and keyword arguments, figure out what
        the query-string options, payload and api arguments are.
        """

        # We need the args to be a list so we can mutate them
        args = list(_args)
        kwargs = copy.deepcopy(_kwargs)

        reqArgs = entry['args']
        routeParams = {}

        query = {}
        payload = None
        kwApiArgs = {}

        paginationHandler = None
        paginationLimit = None

        # There are three formats for calling methods:
        #   1. method(v1, v1, payload)
        #   2. method(payload, k1=v1, k2=v2)
        #   3. method(payload=payload, query=query, params={k1: v1, k2: v2})
        if len(kwargs) == 0:
            if 'input' in entry and len(args) == len(reqArgs) + 1:
                payload = args.pop()
            if len(args) != len(reqArgs):
                log.debug(args)
                log.debug(reqArgs)
                raise exceptions.TaskclusterFailure('Incorrect number of positional arguments')
            log.debug('Using method(v1, v2, payload) calling convention')
        else:
            # We're considering kwargs which are the api route parameters to be
            # called 'flat' because they're top level keys.  We're special
            # casing calls which have only api-arg kwargs and possibly a payload
            # value and handling them directly.
            isFlatKwargs = True
            if len(kwargs) == len(reqArgs):
                for arg in reqArgs:
                    if not kwargs.get(arg, False):
                        isFlatKwargs = False
                        break
                if 'input' in entry and len(args) != 1:
                    isFlatKwargs = False
                if 'input' not in entry and len(args) != 0:
                    isFlatKwargs = False
                else:
                    pass  # We're using payload=, query= and param=
            else:
                isFlatKwargs = False

            # Now we're going to handle the two types of kwargs.  The first is
            # 'flat' ones, which are where the api params
            if isFlatKwargs:
                if 'input' in entry:
                    payload = args.pop()
                kwApiArgs = kwargs
                log.debug('Using method(payload, k1=v1, k2=v2) calling convention')
                warnings.warn(
                    "The method(payload, k1=v1, k2=v2) calling convention will soon be deprecated",
                    PendingDeprecationWarning
                )
            else:
                kwApiArgs = kwargs.get('params', {})
                payload = kwargs.get('payload', None)
                query = kwargs.get('query', {})
                paginationHandler = kwargs.get('paginationHandler', None)
                paginationLimit = kwargs.get('paginationLimit', None)
                log.debug('Using method(payload=payload, query=query, params={k1: v1, k2: v2}) calling convention')

        if 'input' in entry and isinstance(payload, type(None)):
            raise exceptions.TaskclusterFailure('Payload is required')

        # These all need to be rendered down to a string, let's just check that
        # they are up front and fail fast
        for arg in args:
            if not isinstance(arg, six.string_types) and not isinstance(arg, int):
                raise exceptions.TaskclusterFailure(
                    'Positional arg "%s" to %s is not a string or int' % (arg, entry['name']))

        for name, arg in six.iteritems(kwApiArgs):
            if not isinstance(arg, six.string_types) and not isinstance(arg, int):
                raise exceptions.TaskclusterFailure(
                    'KW arg "%s: %s" to %s is not a string or int' % (name, arg, entry['name']))

        if len(args) > 0 and len(kwApiArgs) > 0:
            raise exceptions.TaskclusterFailure('Specify either positional or key word arguments')

        # We know for sure that if we don't give enough arguments that the call
        # should fail.  We don't yet know if we should fail because of two many
        # arguments because we might be overwriting positional ones with kw ones
        if len(reqArgs) > len(args) + len(kwApiArgs):
            raise exceptions.TaskclusterFailure(
                '%s takes %d args, only %d were given' % (
                    entry['name'], len(reqArgs), len(args) + len(kwApiArgs)))

        # We also need to error out when we have more positional args than required
        # because we'll need to go through the lists of provided and required args
        # at the same time.  Not disqualifying early means we'll get IndexErrors if
        # there are more positional arguments than required
        if len(args) > len(reqArgs):
            raise exceptions.TaskclusterFailure('%s called with too many positional args',
                                                entry['name'])

        i = 0
        for arg in args:
            log.debug('Found a positional argument: %s', arg)
            routeParams[reqArgs[i]] = arg
            i += 1

        log.debug('After processing positional arguments, we have: %s', routeParams)

        routeParams.update(kwApiArgs)

        log.debug('After keyword arguments, we have: %s', routeParams)

        if len(reqArgs) != len(routeParams):
            errMsg = '%s takes %s args, %s given' % (
                entry['name'],
                ','.join(reqArgs),
                routeParams.keys())
            log.error(errMsg)
            raise exceptions.TaskclusterFailure(errMsg)

        for reqArg in reqArgs:
            if reqArg not in routeParams:
                errMsg = '%s requires a "%s" argument which was not provided' % (
                    entry['name'], reqArg)
                log.error(errMsg)
                raise exceptions.TaskclusterFailure(errMsg)

        return routeParams, payload, query, paginationHandler, paginationLimit

    def _subArgsInRoute(self, entry, args):
        """ Given a route like "/task/<taskId>/artifacts" and a mapping like
        {"taskId": "12345"}, return a string like "/task/12345/artifacts"
        """

        route = entry['route']

        for arg, val in six.iteritems(args):
            toReplace = "<%s>" % arg
            if toReplace not in route:
                raise exceptions.TaskclusterFailure(
                    'Arg %s not found in route for %s' % (arg, entry['name']))
            val = urllib.parse.quote(str(val).encode("utf-8"), '')
            route = route.replace("<%s>" % arg, val)

        return route.lstrip('/')

    def _hasCredentials(self):
        """ Return True, if credentials is given """
        cred = self.options.get('credentials')
        return (
            cred and
            'clientId' in cred and
            'accessToken' in cred and
            cred['clientId'] and
            cred['accessToken']
        )

    def _makeHttpRequest(self, method, route, payload):
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
                time.sleep(utils.calculateSleepTime(retry))
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
                response = utils.makeSingleHttpRequest(method, url, payload, headers)
            except requests.exceptions.RequestException as rerr:
                if retry < retries:
                    log.warn('Retrying because of: %s' % rerr)
                    continue
                # raise a connection exception
                raise exceptions.TaskclusterConnectionError(
                    "Failed to establish connection",
                    superExc=rerr
                )

            # Handle non 2xx status code and retry if possible
            status = response.status_code
            if status == 204:
                return None

            # Catch retryable errors and go to the beginning of the loop
            # to do the retry
            if 500 <= status and status < 600 and retry < retries:
                log.warn('Retrying because of a %s status code' % status)
                continue

            # Throw errors for non-retryable errors
            if status < 200 or status >= 300:
                data = {}
                try:
                    data = response.json()
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
                return response.json()
            except ValueError:
                return {"response": response}

        # This code-path should be unreachable
        assert False, "Error from last retry should have been raised!"


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
                def apiCall(self, *args, **kwargs):
                    return self._makeApiCall(e, *args, **kwargs)
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
    start: start time of credentials (datetime.datetime)
    expiry: expiration time of credentials, (datetime.datetime)
    scopes: list of scopes granted
    name: credential name (optional)

    Returns a dictionary in the form:
        { 'clientId': str, 'accessToken: str, 'certificate': str}
    """

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
