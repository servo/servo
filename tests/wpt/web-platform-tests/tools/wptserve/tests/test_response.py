from io import BytesIO
from unittest import mock

from wptserve.response import Response


def test_response_status():
    cases = [200, (200, b'OK'), (200, 'OK'), ('200', 'OK')]

    for case in cases:
        handler = mock.Mock()
        handler.wfile = BytesIO()
        request = mock.Mock()
        request.protocol_version = 'HTTP/1.1'
        response = Response(handler, request)

        response.status = case
        expected = case if isinstance(case, tuple) else (case, None)
        if expected[0] == '200':
            expected = (200, expected[1])
        assert response.status == expected
        response.writer.write_status(*response.status)
        assert handler.wfile.getvalue() == b'HTTP/1.1 200 OK\r\n'


def test_response_status_not_string():
    # This behaviour is not documented but kept for backward compatibility.
    handler = mock.Mock()
    request = mock.Mock()
    response = Response(handler, request)
    response.status = (200, 100)
    assert response.status == (200, '100')
