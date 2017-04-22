apiclient
==========

Client class for calling http+json APIs in Python. Requires Python 2.7.

Suports json home pages per:
http://tools.ietf.org/html/draft-nottingham-json-home-03


Installation
------------
Standard python package installation:

       python setup.py install


Usage
-----
Import the apiclient package and instantiate a client.

       import apiclient

       api = apiclient.APIClient('http://api.example.com')

Call APIs:

       result = api.call('foo', var1=arg1, var2=arg2)
       print result.data


APIClient class
---------------
**class apiclient.APIClient(baseURI, version = None, username = None, password = None)**

The APIClient constructor takes the base URI for the api, an optional request version identifier, username and password.

**APIClient.baseURI**

The base URI set in the constructor, read-only.

**APIClient.resourceNames**

A list of available API resource names.

**APIClient.resource(name)**

Get a named APIResource.

**APIClient.setVersion(name, version)**

Set the request version identifier for a specific resource. If not set, the default version identifer will be used.

**APIClient.setAccept(name, mimeType)**

Set the requested Content-Type for a specific resource. If not set, 'application/json' will be used.

**APIClient.get(name, [kwargs])**

Perform an HTTP GET on the named resource. Any named arguments supplied may be used in computing the actual URI to call. Returns an APIResponse or None if the resource name is not known.

**APIClient.postForm(name, payload = None, [kwargs])**

Perform an HTTP POST on the named resource. The payload, if present, may be either a dict or sequence of two-tuples and will be form encoded. Any named arguments supplied may be used in computing the actual URI to call. Returns an APIResponse or None if the resource name is not known.

**APIClient.put(name, payload = None, payloadType = None, [kwargs])**

Perform an HTTP PUT on the named resource. The payload, if present, will be sent to the server using the payloadType Content-Type. The payload must be pre-encoded and will not be processed by the APIClient. Any named arguments supplied may be used in computing the actual URI to call. Returns an APIResponse or None if the resource name is not known.

**APIClient.patch(name, patch = None, [kwargs])**

Perform an HTTP PATCH on the named resource. The patch, if present, will be encoded in JSON and sent to the server as a 'application/json-patch'. Any named arguments supplied may be used in computing the actual URI to call. Returns an APIResponse or None if the resource name is not known.

**APIClient.delete(name, [kwargs])**

Perform an HTTP DELETE on the named resource. Any named arguments supplied may be used in computing the actual URI to call. Returns an APIResponse or None if the resource name is not known.


APIResponse class
-----------------
**APIResponse.status**

The HTTP status code of the response.

**APIResponse.headers**

A dict of HTTP response headers.

**APIResponse.contentType**

The Content-Type of the response.

**APIResponse.encoding**

The encoding of the response.

**APIResponse.data**

The body of the response. If the contentType is json, the data will be decoded into native objects.


APIResource class
-----------------
Describes the properties of an available API resource.

**APIResource.template**

The URITemplate used when calling the resource.

**APIResource.variables**

A dict of variables that may be passed to the resource. Keys are variable names, values are the URI identifier of the variable, if available (see http://tools.ietf.org/html/draft-nottingham-json-home-03#section-3.1 ).

**APIResource.hints**

An APIHints object describing any hints for the resource (see http://tools.ietf.org/html/draft-nottingham-json-home-03#section-4 ).


APIHints class
--------------
**APIHints.httpMethods**

A list of HTTP methods the resource may be called with.

**APIHints.formats**

A dict of formats available for each HTTP method. Keys are HTTP methods, values are a list of Content-Types available.

**APIHints.ranges**

Not yet implemented.

**APIHints.preferences**

Not yet implemented.

**APIHints.preconditions**

Not yet implemented.

**APIHints.auth**

Not yet implemented.

**APIHints.docs**

A URI for documentation for the resource.

**APIHints.status**

The status of the resource.


URITemplate class
-----------------
Parses and expands URITemplates per RFC 6750 (plus a few extensions).

**class uritemplate.URITemplate(template)**

Construct a URITemplate. Raises exceptions if malformed.

**URITemplate.variables**

A set of variables available in the template.

**URITemplate.expand([kwargs])**

Return the expanded template substituting any passed keyword arguments.


Notes
-----
Resource names may be absolute URIs or relative to the base URI of the API.


