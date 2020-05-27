from six import PY3

def get_response(raw_headers, filter_value, filter_name):
    result = b""
    # Type of raw_headers is <httplib.HTTPMessage> in Python 2 and <http.client.HTTPMessage> in
    # Python 3. <http.client.HTTPMessage> doesn't have 'headers" attribute or equivalent
    # [https://bugs.python.org/issue4773].
    # In Python 2, variable raw_headers.headers returns a completely uninterpreted list of lines
    # contained in the header. In Python 3, raw_headers.as_string() returns entire formatted
    # message as a string. Here is to construct an equivalent "headers" variable to support tests
    # in Python 3.
    if PY3:
        header_list = [
            (s + u'\r\n').encode("iso-8859-1") for s in raw_headers.as_string().splitlines() if s
        ]
    else:
        header_list = raw_headers.headers
    for line in header_list:
        if line[-2:] != b'\r\n':
            return b"Syntax error: missing CRLF: " + line
        line = line[:-2]

        if b': ' not in line:
            return b"Syntax error: no colon and space found: " + line
        name, value = line.split(b': ', 1)

        if filter_value:
            if value == filter_value:
                result += name + b","
        elif name.lower() == filter_name:
            result += name + b": " + value + b"\n"
    return result

def main(request, response):
    headers = []
    if "cors" in request.GET:
        headers.append((b"Access-Control-Allow-Origin", b"*"))
        headers.append((b"Access-Control-Allow-Credentials", b"true"))
        headers.append((b"Access-Control-Allow-Methods", b"GET, POST, PUT, FOO"))
        headers.append((b"Access-Control-Allow-Headers", b"x-test, x-foo"))
        headers.append((
            b"Access-Control-Expose-Headers",
            b"x-request-method, x-request-content-type, x-request-query, x-request-content-length"))
    headers.append((b"content-type", b"text/plain"))

    filter_value = request.GET.first(b"filter_value", b"")
    filter_name = request.GET.first(b"filter_name", b"").lower()
    result = get_response(request.raw_headers, filter_value, filter_name)

    return headers, result
