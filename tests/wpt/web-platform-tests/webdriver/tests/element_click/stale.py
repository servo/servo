import pytest
import webdriver

from tests.support.asserts import assert_error
from tests.support.inline import inline


def click_element(session, element):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/click".format(**{
            "session_id": session.session_id,
            "element_id": element.id,
        }))


def test_is_stale(session):
    session.url = inline("<button>foo</button>")
    button = session.find.css("button", all=False)
    session.url = inline("<button>bar</button>")

    response = click_element(session, button)
    assert_error(response, "stale element reference")
