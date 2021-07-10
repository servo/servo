import mock

from wptserve.request import Request, RequestHeaders, MultiDict


class MockHTTPMessage(dict):
    """A minimum (and not completely correctly) mock of HTTPMessage for testing.

    Constructing HTTPMessage is annoying and different in Python 2 and 3. This
    only implements the parts used by RequestHeaders.

    Requirements for construction:
    * Keys are header names and MUST be lower-case.
    * Values are lists of header values (even if there's only one).
    * Keys and values should be native strings to match stdlib's behaviours.
    """
    def __getitem__(self, key):
        assert isinstance(key, str)
        values = dict.__getitem__(self, key.lower())
        assert isinstance(values, list)
        return values[0]

    def get(self, key, default=None):
        try:
            return self[key]
        except KeyError:
            return default

    def getallmatchingheaders(self, key):
        values = dict.__getitem__(self, key.lower())
        return ["{}: {}\n".format(key, v) for v in values]


def test_request_headers_get():
    raw_headers = MockHTTPMessage({
        'x-foo': ['foo'],
        'x-bar': ['bar1', 'bar2'],
    })
    headers = RequestHeaders(raw_headers)
    assert headers['x-foo'] == b'foo'
    assert headers['X-Bar'] == b'bar1, bar2'
    assert headers.get('x-bar') == b'bar1, bar2'


def test_request_headers_encoding():
    raw_headers = MockHTTPMessage({
        'x-foo': ['foo'],
        'x-bar': ['bar1', 'bar2'],
    })
    headers = RequestHeaders(raw_headers)
    assert isinstance(headers['x-foo'], bytes)
    assert isinstance(headers['x-bar'], bytes)
    assert isinstance(headers.get_list('x-bar')[0], bytes)


def test_request_url_from_server_address():
    request_handler = mock.Mock()
    request_handler.server.scheme = 'http'
    request_handler.server.server_address = ('localhost', '8000')
    request_handler.path = '/demo'
    request_handler.headers = MockHTTPMessage()

    request = Request(request_handler)
    assert request.url == 'http://localhost:8000/demo'
    assert isinstance(request.url, str)


def test_request_url_from_host_header():
    request_handler = mock.Mock()
    request_handler.server.scheme = 'http'
    request_handler.server.server_address = ('localhost', '8000')
    request_handler.path = '/demo'
    request_handler.headers = MockHTTPMessage({'host': ['web-platform.test:8001']})

    request = Request(request_handler)
    assert request.url == 'http://web-platform.test:8001/demo'
    assert isinstance(request.url, str)


def test_multidict():
    m = MultiDict()
    m["foo"] = "bar"
    m["bar"] = "baz"
    m.add("foo", "baz")
    m.add("baz", "qux")

    assert m["foo"] == "bar"
    assert m.get("foo") == "bar"
    assert m["bar"] == "baz"
    assert m.get("bar") == "baz"
    assert m["baz"] == "qux"
    assert m.get("baz") == "qux"

    assert m.first("foo") == "bar"
    assert m.last("foo") == "baz"
    assert m.get_list("foo") == ["bar", "baz"]
    assert m.get_list("non_existent") == []

    assert m.get("non_existent") is None
    try:
        m["non_existent"]
        assert False, "An exception should be raised"
    except KeyError:
        pass
