import base64

import pytest

from six import ensure_binary

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def do_print(session, options):
    return session.transport.send(
        "POST", "session/{session_id}/print".format(**vars(session)),
        options)


def assert_pdf(data):
    assert data.startswith(b"%PDF-"), "Decoded data starts with the PDF signature"
    assert data.endswith(b"%%EOF\n"), "Decoded data ends with the EOF flag"


def test_no_top_browsing_context(session, closed_window):
    response = do_print(session, {})
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = do_print(session, {})
    value = assert_success(response)
    pdf = base64.decodestring(value)
    assert_pdf(pdf)


def test_html_document(session):
    session.url = inline("Test")

    response = do_print(session, {
        "page": {"width": 10,
                 "height": 20},
        "shrinkToFit": False
    })
    value = assert_success(response)
    pdf = base64.decodestring(ensure_binary(value))
    # TODO: Test that the output is reasonable
    assert_pdf(pdf)


@pytest.mark.parametrize("options", [{"orientation": 0},
                                     {"orientation": "foo"},
                                     {"scale": "1"},
                                     {"scale": 3},
                                     {"scale": 0.01},
                                     {"margin": {"top": "1"}},
                                     {"margin": {"bottom": -1}},
                                     {"page": {"height": False}},
                                     {"shrinkToFit": "false"}])
def test_invalid(session, options):
    response = do_print(session, options)
    assert_error(response, "invalid argument")
