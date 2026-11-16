import pytest

from tests.support.classic.asserts import assert_success


def is_element_selected(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/selected".format(
            session_id=session.session_id,
            element_id=element_id))


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
@pytest.mark.parametrize("status,expected", [
    ("checked", True),
    ("", False),
], ids=["checked", "unchecked"])
def test_element_selected_in_frame(session, inline, iframe, status, expected, domain):
    session.url = inline(iframe("<input type=checkbox id='x' {}>".format(status), domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css("#x", all=False)
    result = is_element_selected(session, element.id)
    assert_success(result, expected)
