# mypy: allow-untyped-defs

import io
from unittest import mock

import requests

from tools.wpt import httputils


def make_response(content):
    response = requests.models.Response()
    response.status_code = 200
    response.raw = io.BytesIO(content)
    return response


@mock.patch("tools.wpt.httputils.requests.get")
def test_get_download_to_descriptor_passes_timeout(mocked_get):
    mocked_get.return_value = make_response(b"data")
    fd = io.BytesIO()

    httputils.get_download_to_descriptor(fd, "https://example.test/", timeout=(1, 2))

    mocked_get.assert_called_once_with("https://example.test/", stream=True, timeout=(1, 2))
    assert fd.getvalue() == b"data"


@mock.patch("tools.wpt.httputils.time.sleep")
@mock.patch("tools.wpt.httputils.requests.get")
def test_get_download_to_descriptor_retries_timeout(mocked_get, mocked_sleep):
    mocked_get.side_effect = [requests.exceptions.ReadTimeout(), make_response(b"data")]
    fd = io.BytesIO()

    httputils.get_download_to_descriptor(fd, "https://example.test/")

    assert mocked_get.call_count == 2
    assert fd.getvalue() == b"data"
