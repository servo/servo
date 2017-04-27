# coding=utf-8
#
#  Copyright © 2013 Hewlett-Packard Development Company, L.P.
#
#  This work is distributed under the W3C® Software License [1]
#  in the hope that it will be useful, but WITHOUT ANY
#  WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
#
#  [1] http://www.w3.org/Consortium/Legal/2002/copyright-software-20021231
#

# Process URI templates per http://tools.ietf.org/html/rfc6570


import urllib2
import urlparse
import json
import base64
import contextlib
import collections
import UserString

import uritemplate

class MimeType(UserString.MutableString):
    def __init__(self, mimeType):
        UserString.MutableString.__init__(self, mimeType)
        self._type = None
        self._subtype = None
        self._structure = None

        slashIndex = mimeType.find('/')
        if (-1 < slashIndex):
            self._type = mimeType[:slashIndex]
            mimeType = mimeType[slashIndex + 1:]
            plusIndex = mimeType.find('+')
            if (-1 < plusIndex):
                self._subtype = mimeType[:plusIndex]
                self._structure = mimeType[plusIndex + 1:]
            else:
                self._structure = mimeType
        else:
            self._type = mimeType

    def _update(self):
        if (self._structure):
            if (self._subtype):
                self.data = self._type + '/' + self._subtype + '+' + self._structure
            else:
                self.data = self._type + '/' + self._structure
        else:
            self.data = self._type

    def set(self, type, structure, subtype = None):
        self._type = type
        self._subtype = subtype
        self._structure = structure
        self._update()

    @property
    def type(self):
        return self._type
    
    @type.setter
    def type(self, value):
        self._type = value
        self._update()

    @property
    def subtype(self):
        return self._subtype

    @subtype.setter
    def subtype(self, value):
        self._subtype = value
        self._update()

    @property
    def structure(self):
        return self._structure

    @structure.setter
    def structure(self, value):
        self._structure = value
        self._update()


class APIResponse(object):
    def __init__(self, response):
        self.status = response.getcode() if (response) else 0
        self.headers = response.info() if (response) else {}
        self.data = response.read() if (200 == self.status) else None

        if (self.data and
            (('json' == self.contentType.structure) or ('json-home' == self.contentType.structure))):
            try:
                self.data = json.loads(self.data, object_pairs_hook = collections.OrderedDict)
            except:
                pass

    @property
    def contentType(self):
        contentType = self.headers.get('content-type') if (self.headers) else None
        return MimeType(contentType.split(';')[0]) if (contentType and (';' in contentType)) else MimeType(contentType)
    
    @property
    def encoding(self):
        contentType = self.headers.get('content-type') if (self.headers) else None
        if (contentType and (';' in contentType)):
            encoding = contentType.split(';', 1)[1]
            if ('=' in encoding):
                return encoding.split('=', 1)[1].strip()
        return 'utf-8'
    

class APIHints(object):
    def __init__(self, data):
        self.httpMethods = [method.upper() for method in data['allow'] if method] if ('allow' in data) else ['GET']
        self.formats = {}
        formats = [MimeType(format) for format in data['formats']] if ('formats' in data) else []
        if (formats):
            if ('GET' in self.httpMethods):
                self.formats['GET'] = formats
            if ('PUT' in self.httpMethods):
                self.formats['PUT'] = formats
    
        if (('PATCH' in self.httpMethods) and ('accept-patch' in data)):
            self.formats['PATCH'] = [MimeType(format) for format in data['accept-patch']]
        if (('POST' in self.httpMethods) and ('accept-post' in data)):
            self.formats['POST'] = [MimeType(format) for format in data['accept-post']]
    
        # TODO: ranges from 'accept-ranges'; preferece tokens from 'accept-prefer';
        #       preconditions from 'precondition-req'; auth from 'auth-req'
        self.ranges = None
        self.preferences = None
        self.preconditions = None
        self.auth = None

        self.docs = data.get('docs')
        self.status = data.get('status')


class APIResource(object):
    def __init__(self, baseURI, uri, variables = None, hints = None):
        try:
            self.template = uritemplate.URITemplate(urlparse.urljoin(baseURI, uri))
            if (variables):
                self.variables = {variable: urlparse.urljoin(baseURI, variables[variable]) for variable in variables}
            else:
                self.variables = {variable: '' for variable in self.template.variables}
            self.hints = hints
        except Exception as e:
            self.template = uritemplate.URITemplate('')
            self.variables = {}
            self.hints = None


class APIClient(object):
    def __init__(self, baseURI, version = None, username = None, password = None):
        self._baseURI = baseURI
        self.defaultVersion = version
        self.defaultAccept = 'application/json'
        self.username = username
        self.password = password
        self._resources = {}
        self._versions = {}
        self._accepts = {}
        
        self._loadHome()
    

    @property
    def baseURI(self):
        return self._baseURI
    
    def _loadHome(self):
        home = self._callURI('GET', self.baseURI, 'application/home+json, application/json-home, application/json')
        if (home):
            if ('application/json' == home.contentType):
                for name in home.data:
                    apiKey = urlparse.urljoin(self.baseURI, name)
                    self._resources[apiKey] = APIResource(self.baseURI, home.data[name])
            elif (('application/home+json' == home.contentType) or
                  ('application/json-home' == home.contentType)):
                resources =  home.data.get('resources')
                if (resources):
                    for name in resources:
                        apiKey = urlparse.urljoin(self.baseURI, name)
                        data = resources[name]
                        uri = data['href'] if ('href' in data) else data.get('href-template')
                        variables = data.get('href-vars')
                        hints = APIHints(data['hints']) if ('hints' in data) else None
                        self._resources[apiKey] = APIResource(self.baseURI, uri, variables, hints)


    def relativeURI(self, uri):
        if (uri.startswith(self.baseURI)):
            relative = uri[len(self.baseURI):]
            if (relative.startswith('/') and not self.baseURI.endswith('/')):
                relative = relative[1:]
            return relative
        return uri

    @property
    def resourceNames(self):
        return [self.relativeURI(apiKey) for apiKey in self._resources]

    def resource(self, name):
        return self._resources.get(urlparse.urljoin(self.baseURI, name))
    
    def addResource(self, name, uri):
        resource = APIResource(self.baseURI, uri)
        apiKey = urlparse.urljoin(self.baseURI, name)
        self._resources[apiKey] = resource

    def _accept(self, resource):
        version = None
        if (api and (api in self._versions)):
            version = self._versions[api]
        if (not version):
            version = self.defaultVersion
        return ('application/' + version + '+json, application/json') if (version) else 'application/json'

    def _callURI(self, method, uri, accept, payload = None, payloadType = None):
        try:
            request = urllib2.Request(uri, data = payload, headers = { 'Accept' : accept })
            if (self.username and self.password):
                request.add_header('Authorization', b'Basic ' + base64.b64encode(self.username + b':' + self.password))
            if (payload and payloadType):
                request.add_header('Content-Type', payloadType)
            request.get_method = lambda: method
            
            with contextlib.closing(urllib2.urlopen(request)) as response:
                return APIResponse(response)
        except Exception as e:
            pass
        return None
    
    def _call(self, method, name, arguments, payload = None, payloadType = None):
        apiKey = urlparse.urljoin(self.baseURI, name)
        resource = self._resources.get(apiKey)
        
        if (resource):
            uri = resource.template.expand(**arguments)
            if (uri):
                version = self._versions.get(apiKey) if (apiKey in self._versions) else self.defaultVersion
                accept = MimeType(self._accepts(apiKey) if (apiKey in self._accepts) else self.defaultAccept)
                if (version):
                    accept.subtype = version
                return self._callURI(method, uri, accept, payload, payloadType)
        return None
    
    def setVersion(self, name, version):
        apiKey = urlparse.urljoin(self.baseURI, name)
        self._versions[apiKey] = version

    def setAccept(self, name, mimeType):
        apiKey = urlparse.urljoin(self.baseURI, name)
        self._accepts[apiKey] = mimeType

    def get(self, name, **kwargs):
        return self._call('GET', name, kwargs)
    
    def post(self, name, payload = None, payloadType = None, **kwargs):
        return self._call('POST', name, kwargs, payload, payloadType)

    def postForm(self, name, payload = None, **kwargs):
        return self._call('POST', name, kwargs, urllib.urlencode(payload), 'application/x-www-form-urlencoded')

    def postJSON(self, name, payload = None, **kwargs):
        return self._call('POST', name, kwargs, json.dumps(payload), 'application/json')

    def put(self, name, payload = None, payloadType = None, **kwargs):
        return self._call('PUT', name, kwargs, payload, payloadType)

    def patch(self, name, patch = None, **kwargs):
        return self._call('PATCH', name, kwargs, json.dumps(patch), 'application/json-patch')

    def delete(self, name, **kwargs):
        return self._call('DELETE', name, kwargs)


