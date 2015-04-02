import base64
import cgi
import Cookie
import StringIO
import tempfile
import urlparse

import stash
from utils import HTTPException

missing = object()


class Server(object):
    """Data about the server environment

    .. attribute:: config

    Environment configuration information with information about the
    various servers running, their hostnames and ports.

    .. attribute:: stash

    Stash object holding state stored on the server between requests.

    """
    config = None

    def __init__(self, request):
        self.stash = stash.Stash(request.url_parts.path)


class InputFile(object):
    max_buffer_size = 1024*1024

    def __init__(self, rfile, length):
        """File-like object used to provide a seekable view of request body data"""
        self._file = rfile
        self.length = length

        self._file_position = 0

        if length > self.max_buffer_size:
            self._buf = tempfile.TemporaryFile(mode="rw+b")
        else:
            self._buf = StringIO.StringIO()

    @property
    def _buf_position(self):
        rv = self._buf.tell()
        assert rv <= self._file_position
        return rv

    def read(self, bytes=-1):
        assert self._buf_position <= self._file_position

        if bytes < 0:
            bytes = self.length - self._buf_position
        bytes_remaining = min(bytes, self.length - self._buf_position)

        if bytes_remaining == 0:
            return ""

        if self._buf_position != self._file_position:
            buf_bytes = min(bytes_remaining, self._file_position - self._buf_position)
            old_data = self._buf.read(buf_bytes)
            bytes_remaining -= buf_bytes
        else:
            old_data = ""

        assert self._buf_position == self._file_position, (
            "Before reading buffer position (%i) didn't match file position (%i)" %
            (self._buf_position, self._file_position))
        new_data = self._file.read(bytes_remaining)
        self._buf.write(new_data)
        self._file_position += bytes_remaining
        assert self._buf_position == self._file_position, (
            "After reading buffer position (%i) didn't match file position (%i)" %
            (self._buf_position, self._file_position))

        return old_data + new_data

    def tell(self):
        return self._buf_position

    def seek(self, offset):
        if offset > self.length or offset < 0:
            raise ValueError
        if offset <= self._file_position:
            self._buf.seek(offset)
        else:
            self.read(offset - self._file_position)

    def readline(self, max_bytes=None):
        if max_bytes is None:
            max_bytes = self.length - self._buf_position

        if self._buf_position < self._file_position:
            data = self._buf.readline(max_bytes)
            if data.endswith("\n") or len(data) == max_bytes:
                return data
        else:
            data = ""

        assert self._buf_position == self._file_position

        initial_position = self._file_position
        found = False
        buf = []
        max_bytes -= len(data)
        while not found:
            readahead = self.read(min(2, max_bytes))
            max_bytes -= len(readahead)
            for i, c in enumerate(readahead):
                if c == "\n":
                    buf.append(readahead[:i+1])
                    found = True
                    break
            if not found:
                buf.append(readahead)
            if not readahead or not max_bytes:
                break
        new_data = "".join(buf)
        data += new_data
        self.seek(initial_position + len(new_data))
        return data

    def readlines(self):
        rv = []
        while True:
            data = self.readline()
            if data:
                rv.append(data)
            else:
                break
        return rv

    def next(self):
        data = self.readline()
        if data:
            return data
        else:
            raise StopIteration

    def __iter__(self):
        return self


class Request(object):
    """Object representing a HTTP request.

    .. attribute:: doc_root

    The local directory to use as a base when resolving paths

    .. attribute:: route_match

    Regexp match object from matching the request path to the route
    selected for the request.

    .. attribute:: protocol_version

    HTTP version specified in the request.

    .. attribute:: method

    HTTP method in the request.

    .. attribute:: request_path

    Request path as it appears in the HTTP request.

    .. attribute:: url

    Absolute URL for the request.

    .. attribute:: headers

    List of request headers.

    .. attribute:: raw_input

    File-like object representing the body of the request.

    .. attribute:: url_parts

    Parts of the requested URL as obtained by urlparse.urlsplit(path)

    .. attribute:: request_line

    Raw request line

    .. attribute:: headers

    RequestHeaders object providing a dictionary-like representation of
    the request headers.

    .. attribute:: body

    Request body as a string

    .. attribute:: GET

    MultiDict representing the parameters supplied with the request.
    Note that these may be present on non-GET requests; the name is
    chosen to be familiar to users of other systems such as PHP.

    .. attribute:: POST

    MultiDict representing the request body parameters. Most parameters
    are present as string values, but file uploads have file-like
    values.

    .. attribute:: cookies

    Cookies object representing cookies sent with the request with a
    dictionary-like interface.

    .. attribute:: auth

    Object with username and password properties representing any
    credentials supplied using HTTP authentication.

    .. attribute:: server

    Server object containing information about the server environment.
    """

    def __init__(self, request_handler):
        self.doc_root = request_handler.server.router.doc_root
        self.route_match = None  # Set by the router

        self.protocol_version = request_handler.protocol_version
        self.method = request_handler.command

        scheme = request_handler.server.scheme
        host = request_handler.headers.get("Host")
        port = request_handler.server.server_address[1]

        if host is None:
            host = request_handler.server.server_address[0]
        else:
            if ":" in host:
                host, port = host.split(":", 1)

        self.request_path = request_handler.path

        if self.request_path.startswith(scheme + "://"):
            self.url = request_handler.path
        else:
            self.url = "%s://%s:%s%s" % (scheme,
                                      host,
                                      port,
                                      self.request_path)
        self.url_parts = urlparse.urlsplit(self.url)

        self._raw_headers = request_handler.headers

        self.request_line = request_handler.raw_requestline

        self._headers = None

        self.raw_input = InputFile(request_handler.rfile,
                                   int(self.headers.get("Content-Length", 0)))
        self._body = None

        self._GET = None
        self._POST = None
        self._cookies = None
        self._auth = None

        self.server = Server(self)

    def __repr__(self):
        return "<Request %s %s>" % (self.method, self.url)

    @property
    def GET(self):
        if self._GET is None:
            params = urlparse.parse_qsl(self.url_parts.query, keep_blank_values=True)
            self._GET = MultiDict()
            for key, value in params:
                self._GET.add(key, value)
        return self._GET

    @property
    def POST(self):
        if self._POST is None:
            #Work out the post parameters
            pos = self.raw_input.tell()
            self.raw_input.seek(0)
            fs = cgi.FieldStorage(fp=self.raw_input,
                                  environ={"REQUEST_METHOD": self.method},
                                  headers=self.headers,
                                  keep_blank_values=True)
            self._POST = MultiDict.from_field_storage(fs)
            self.raw_input.seek(pos)
        return self._POST

    @property
    def cookies(self):
        if self._cookies is None:
            parser = Cookie.BaseCookie()
            cookie_headers = self.headers.get("cookie", "")
            parser.load(cookie_headers)
            cookies = Cookies()
            for key, value in parser.iteritems():
                cookies[key] = CookieValue(value)
            self._cookies = cookies
        return self._cookies

    @property
    def headers(self):
        if self._headers is None:
            self._headers = RequestHeaders(self._raw_headers)
        return self._headers

    @property
    def body(self):
        if self._body is None:
            pos = self.raw_input.tell()
            self.raw_input.seek(0)
            self._body = self.raw_input.read()
            self.raw_input.seek(pos)
        return self._body

    @property
    def auth(self):
        if self._auth is None:
            self._auth = Authentication(self.headers)
        return self._auth


class RequestHeaders(dict):
    """Dictionary-like API for accessing request headers."""
    def __init__(self, items):
        for key, value in zip(items.keys(), items.values()):
            key = key.lower()
            if key in self:
                self[key].append(value)
            else:
                dict.__setitem__(self, key, [value])

    def __getitem__(self, key):
        """Get all headers of a certain (case-insensitive) name. If there is
        more than one, the values are returned comma separated"""
        values = dict.__getitem__(self, key.lower())
        if len(values) == 1:
            return values[0]
        else:
            return ", ".join(values)

    def __setitem__(self, name, value):
        raise Exception

    def get(self, key, default=None):
        """Get a string representing all headers with a particular value,
        with multiple headers separated by a comma. If no header is found
        return a default value

        :param key: The header name to look up (case-insensitive)
        :param default: The value to return in the case of no match
        """
        try:
            return self[key]
        except KeyError:
            return default

    def get_list(self, key, default=missing):
        """Get all the header values for a particular field name as
        a list"""
        try:
            return dict.__getitem__(self, key.lower())
        except KeyError:
            if default is not missing:
                return default
            else:
                raise

    def __contains__(self, key):
        return dict.__contains__(self, key.lower())

    def iteritems(self):
        for item in self:
            yield item, self[item]

    def itervalues(self):
        for item in self:
            yield self[item]

class CookieValue(object):
    """Representation of cookies.

    Note that cookies are considered read-only and the string value
    of the cookie will not change if you update the field values.
    However this is not enforced.

    .. attribute:: key

    The name of the cookie.

    .. attribute:: value

    The value of the cookie

    .. attribute:: expires

    The expiry date of the cookie

    .. attribute:: path

    The path of the cookie

    .. attribute:: comment

    The comment of the cookie.

    .. attribute:: domain

    The domain with which the cookie is associated

    .. attribute:: max_age

    The max-age value of the cookie.

    .. attribute:: secure

    Whether the cookie is marked as secure

    .. attribute:: httponly

    Whether the cookie is marked as httponly

    """
    def __init__(self, morsel):
        self.key = morsel.key
        self.value = morsel.value

        for attr in ["expires", "path",
                     "comment", "domain", "max-age",
                     "secure", "version", "httponly"]:
            setattr(self, attr.replace("-", "_"), morsel[attr])

        self._str = morsel.OutputString()

    def __str__(self):
        return self._str

    def __repr__(self):
        return self._str

    def __eq__(self, other):
        """Equality comparison for cookies. Compares to other cookies
        based on value alone and on non-cookies based on the equality
        of self.value with the other object so that a cookie with value
        "ham" compares equal to the string "ham"
        """
        if hasattr(other, "value"):
            return self.value == other.value
        return self.value == other


class MultiDict(dict):
    """Dictionary type that holds multiple values for each
    key"""
    #TODO: this should perhaps also order the keys
    def __init__(self):
        pass

    def __setitem__(self, name, value):
        dict.__setitem__(self, name, [value])

    def add(self, name, value):
        if name in self:
            dict.__getitem__(self, name).append(value)
        else:
            dict.__setitem__(self, name, [value])

    def __getitem__(self, key):
        """Get the first value with a given key"""
        #TODO: should this instead be the last value?
        return self.first(key)

    def first(self, key, default=missing):
        """Get the first value with a given key

        :param key: The key to lookup
        :param default: The default to return if key is
                        not found (throws if nothing is
                        specified)
        """
        if key in self and dict.__getitem__(self, key):
            return dict.__getitem__(self, key)[0]
        elif default is not missing:
            return default
        raise KeyError

    def last(self, key, default=missing):
        """Get the last value with a given key

        :param key: The key to lookup
        :param default: The default to return if key is
                        not found (throws if nothing is
                        specified)
        """
        if key in self and dict.__getitem__(self, key):
            return dict.__getitem__(self, key)[-1]
        elif default is not missing:
            return default
        raise KeyError

    def get_list(self, key):
        """Get all values with a given key as a list

        :param key: The key to lookup
        """
        return dict.__getitem__(self, key)

    @classmethod
    def from_field_storage(cls, fs):
        self = cls()
        if fs.list is None:
            return self
        for key in fs:
            values = fs[key]
            if not isinstance(values, list):
                values = [values]

            for value in values:
                if value.filename:
                    value = value
                else:
                    value = value.value
                self.add(key, value)
        return self


class Cookies(MultiDict):
    """MultiDict specialised for Cookie values"""
    def __init__(self):
        pass

    def __getitem__(self, key):
        return self.last(key)


class Authentication(object):
    """Object for dealing with HTTP Authentication

    .. attribute:: username

    The username supplied in the HTTP Authorization
    header, or None

    .. attribute:: password

    The password supplied in the HTTP Authorization
    header, or None
    """
    def __init__(self, headers):
        self.username = None
        self.password = None

        auth_schemes = {"Basic": self.decode_basic}

        if "authorization" in headers:
            header = headers.get("authorization")
            auth_type, data = header.split(" ", 1)
            if auth_type in auth_schemes:
                self.username, self.password = auth_schemes[auth_type](data)
            else:
                raise HTTPException(400, "Unsupported authentication scheme %s" % auth_type)

    def decode_basic(self, data):
        decoded_data = base64.decodestring(data)
        return decoded_data.split(":", 1)
