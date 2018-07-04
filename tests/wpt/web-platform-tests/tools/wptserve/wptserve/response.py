from collections import OrderedDict
from datetime import datetime, timedelta
from six.moves.http_cookies import BaseCookie, Morsel
import json
import uuid
import socket

from .constants import response_codes
from .logger import get_logger

from six import binary_type, text_type, itervalues

missing = object()

class Response(object):
    """Object representing the response to a HTTP request

    :param handler: RequestHandler being used for this response
    :param request: Request that this is the response for

    .. attribute:: request

       Request associated with this Response.

    .. attribute:: encoding

       The encoding to use when converting unicode to strings for output.

    .. attribute:: add_required_headers

       Boolean indicating whether mandatory headers should be added to the
       response.

    .. attribute:: send_body_for_head_request

       Boolean, default False, indicating whether the body content should be
       sent when the request method is HEAD.

    .. attribute:: explicit_flush

       Boolean indicating whether output should be flushed automatically or only
       when requested.

    .. attribute:: writer

       The ResponseWriter for this response

    .. attribute:: status

       Status tuple (code, message). Can be set to an integer, in which case the
       message part is filled in automatically, or a tuple.

    .. attribute:: headers

       List of HTTP headers to send with the response. Each item in the list is a
       tuple of (name, value).

    .. attribute:: content

       The body of the response. This can either be a string or a iterable of response
       parts. If it is an iterable, any item may be a string or a function of zero
       parameters which, when called, returns a string."""

    def __init__(self, handler, request):
        self.request = request
        self.encoding = "utf8"

        self.add_required_headers = True
        self.send_body_for_head_request = False
        self.explicit_flush = False
        self.close_connection = False

        self.writer = ResponseWriter(handler, self)

        self._status = (200, None)
        self.headers = ResponseHeaders()
        self.content = []

        self.logger = get_logger()

    @property
    def status(self):
        return self._status

    @status.setter
    def status(self, value):
        if hasattr(value, "__len__"):
            if len(value) != 2:
                raise ValueError
            else:
                self._status = (int(value[0]), str(value[1]))
        else:
            self._status = (int(value), None)

    def set_cookie(self, name, value, path="/", domain=None, max_age=None,
                   expires=None, secure=False, httponly=False, comment=None):
        """Set a cookie to be sent with a Set-Cookie header in the
        response

        :param name: String name of the cookie
        :param value: String value of the cookie
        :param max_age: datetime.timedelta int representing the time (in seconds)
                        until the cookie expires
        :param path: String path to which the cookie applies
        :param domain: String domain to which the cookie applies
        :param secure: Boolean indicating whether the cookie is marked as secure
        :param httponly: Boolean indicating whether the cookie is marked as
                         HTTP Only
        :param comment: String comment
        :param expires: datetime.datetime or datetime.timedelta indicating a
                        time or interval from now when the cookie expires

        """
        days = dict((i+1, name) for i, name in enumerate(["jan", "feb", "mar",
                                                          "apr", "may", "jun",
                                                          "jul", "aug", "sep",
                                                          "oct", "nov", "dec"]))
        if value is None:
            value = ''
            max_age = 0
            expires = timedelta(days=-1)

        if isinstance(expires, timedelta):
            expires = datetime.utcnow() + expires

        if expires is not None:
            expires_str = expires.strftime("%d %%s %Y %H:%M:%S GMT")
            expires_str = expires_str % days[expires.month]
            expires = expires_str

        if max_age is not None:
            if hasattr(max_age, "total_seconds"):
                max_age = int(max_age.total_seconds())
            max_age = "%.0d" % max_age

        m = Morsel()

        def maybe_set(key, value):
            if value is not None and value is not False:
                m[key] = value

        m.set(name, value, value)
        maybe_set("path", path)
        maybe_set("domain", domain)
        maybe_set("comment", comment)
        maybe_set("expires", expires)
        maybe_set("max-age", max_age)
        maybe_set("secure", secure)
        maybe_set("httponly", httponly)

        self.headers.append("Set-Cookie", m.OutputString())

    def unset_cookie(self, name):
        """Remove a cookie from those that are being sent with the response"""
        cookies = self.headers.get("Set-Cookie")
        parser = BaseCookie()
        for cookie in cookies:
            parser.load(cookie)

        if name in parser.keys():
            del self.headers["Set-Cookie"]
            for m in parser.values():
                if m.key != name:
                    self.headers.append(("Set-Cookie", m.OutputString()))

    def delete_cookie(self, name, path="/", domain=None):
        """Delete a cookie on the client by setting it to the empty string
        and to expire in the past"""
        self.set_cookie(name, None, path=path, domain=domain, max_age=0,
                        expires=timedelta(days=-1))

    def iter_content(self, read_file=False):
        """Iterator returning chunks of response body content.

        If any part of the content is a function, this will be called
        and the resulting value (if any) returned.

        :param read_file: - boolean controlling the behaviour when content
        is a file handle. When set to False the handle will be returned directly
        allowing the file to be passed to the output in small chunks. When set to
        True, the entire content of the file will be returned as a string facilitating
        non-streaming operations like template substitution.
        """
        if isinstance(self.content, (binary_type, text_type)):
            yield self.content
        elif hasattr(self.content, "read"):
            if read_file:
                yield self.content.read()
            else:
                yield self.content
        else:
            for item in self.content:
                if hasattr(item, "__call__"):
                    value = item()
                else:
                    value = item
                if value:
                    yield value

    def write_status_headers(self):
        """Write out the status line and headers for the response"""
        self.writer.write_status(*self.status)
        for item in self.headers:
            self.writer.write_header(*item)
        self.writer.end_headers()

    def write_content(self):
        """Write out the response content"""
        if self.request.method != "HEAD" or self.send_body_for_head_request:
            for item in self.iter_content():
                self.writer.write_content(item)

    def write(self):
        """Write the whole response"""
        self.write_status_headers()
        self.write_content()

    def set_error(self, code, message=""):
        """Set the response status headers and body to indicate an
        error"""
        err = {"code": code,
               "message": message}
        data = json.dumps({"error": err})
        self.status = code
        self.headers = [("Content-Type", "application/json"),
                        ("Content-Length", len(data))]
        self.content = data
        if code == 500:
            self.logger.error(message)


class MultipartContent(object):
    def __init__(self, boundary=None, default_content_type=None):
        self.items = []
        if boundary is None:
            boundary = str(uuid.uuid4())
        self.boundary = boundary
        self.default_content_type = default_content_type

    def __call__(self):
        boundary = "--" + self.boundary
        rv = ["", boundary]
        for item in self.items:
            rv.append(str(item))
            rv.append(boundary)
        rv[-1] += "--"
        return "\r\n".join(rv)

    def append_part(self, data, content_type=None, headers=None):
        if content_type is None:
            content_type = self.default_content_type
        self.items.append(MultipartPart(data, content_type, headers))

    def __iter__(self):
        #This is hackish; when writing the response we need an iterable
        #or a string. For a multipart/byterange response we want an
        #iterable that contains a single callable; the MultipartContent
        #object itself
        yield self


class MultipartPart(object):
    def __init__(self, data, content_type=None, headers=None):
        self.headers = ResponseHeaders()

        if content_type is not None:
            self.headers.set("Content-Type", content_type)

        if headers is not None:
            for name, value in headers:
                if name.lower() == "content-type":
                    func = self.headers.set
                else:
                    func = self.headers.append
                func(name, value)

        self.data = data

    def __str__(self):
        rv = []
        for item in self.headers:
            rv.append("%s: %s" % item)
        rv.append("")
        rv.append(self.data)
        return "\r\n".join(rv)


class ResponseHeaders(object):
    """Dictionary-like object holding the headers for the response"""
    def __init__(self):
        self.data = OrderedDict()

    def set(self, key, value):
        """Set a header to a specific value, overwriting any previous header
        with the same name

        :param key: Name of the header to set
        :param value: Value to set the header to
        """
        self.data[key.lower()] = (key, [value])

    def append(self, key, value):
        """Add a new header with a given name, not overwriting any existing
        headers with the same name

        :param key: Name of the header to add
        :param value: Value to set for the header
        """
        if key.lower() in self.data:
            self.data[key.lower()][1].append(value)
        else:
            self.set(key, value)

    def get(self, key, default=missing):
        """Get the set values for a particular header."""
        try:
            return self[key]
        except KeyError:
            if default is missing:
                return []
            return default

    def __getitem__(self, key):
        """Get a list of values for a particular header

        """
        return self.data[key.lower()][1]

    def __delitem__(self, key):
        del self.data[key.lower()]

    def __contains__(self, key):
        return key.lower() in self.data

    def __setitem__(self, key, value):
        self.set(key, value)

    def __iter__(self):
        for key, values in itervalues(self.data):
            for value in values:
                yield key, value

    def items(self):
        return list(self)

    def update(self, items_iter):
        for name, value in items_iter:
            self.append(name, value)

    def __repr__(self):
        return repr(self.data)


class ResponseWriter(object):
    """Object providing an API to write out a HTTP response.

    :param handler: The RequestHandler being used.
    :param response: The Response associated with this writer.

    After each part of the response is written, the output is
    flushed unless response.explicit_flush is False, in which case
    the user must call .flush() explicitly."""
    def __init__(self, handler, response):
        self._wfile = handler.wfile
        self._response = response
        self._handler = handler
        self._headers_seen = set()
        self._headers_complete = False
        self.content_written = False
        self.request = response.request
        self.file_chunk_size = 32 * 1024

    def write_status(self, code, message=None):
        """Write out the status line of a response.

        :param code: The integer status code of the response.
        :param message: The message of the response. Defaults to the message commonly used
                        with the status code."""
        if message is None:
            if code in response_codes:
                message = response_codes[code][0]
            else:
                message = ''
        self.write("%s %d %s\r\n" %
                   (self._response.request.protocol_version, code, message))

    def write_header(self, name, value):
        """Write out a single header for the response.

        :param name: Name of the header field
        :param value: Value of the header field
        """
        self._headers_seen.add(name.lower())
        self.write("%s: %s\r\n" % (name, value))
        if not self._response.explicit_flush:
            self.flush()

    def write_default_headers(self):
        for name, f in [("Server", self._handler.version_string),
                        ("Date", self._handler.date_time_string)]:
            if name.lower() not in self._headers_seen:
                self.write_header(name, f())

        if (isinstance(self._response.content, (binary_type, text_type)) and
            "content-length" not in self._headers_seen):
            #Would be nice to avoid double-encoding here
            self.write_header("Content-Length", len(self.encode(self._response.content)))

    def end_headers(self):
        """Finish writing headers and write the separator.

        Unless add_required_headers on the response is False,
        this will also add HTTP-mandated headers that have not yet been supplied
        to the response headers"""

        if self._response.add_required_headers:
            self.write_default_headers()

        self.write("\r\n")
        if "content-length" not in self._headers_seen:
            self._response.close_connection = True
        if not self._response.explicit_flush:
            self.flush()
        self._headers_complete = True

    def write_content(self, data):
        """Write the body of the response."""
        if isinstance(data, (text_type, binary_type)):
            # Deliberately allows both text and binary types. See `self.encode`.
            self.write(data)
        else:
            self.write_content_file(data)
        if not self._response.explicit_flush:
            self.flush()

    def write(self, data):
        """Write directly to the response, converting unicode to bytes
        according to response.encoding. Does not flush."""
        self.content_written = True
        try:
            self._wfile.write(self.encode(data))
        except socket.error:
            # This can happen if the socket got closed by the remote end
            pass

    def write_content_file(self, data):
        """Write a file-like object directly to the response in chunks.
        Does not flush."""
        self.content_written = True
        while True:
            buf = data.read(self.file_chunk_size)
            if not buf:
                break
            try:
                self._wfile.write(buf)
            except socket.error:
                break
        data.close()

    def encode(self, data):
        """Convert unicode to bytes according to response.encoding."""
        if isinstance(data, binary_type):
            return data
        elif isinstance(data, text_type):
            return data.encode(self._response.encoding)
        else:
            raise ValueError

    def flush(self):
        """Flush the output. Returns False if the flush failed due to
        the socket being closed by the remote end."""
        try:
            self._wfile.flush()
            return True
        except socket.error:
            # This can happen if the socket got closed by the remote end
            return False
