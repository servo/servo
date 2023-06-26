from tests.support.asserts import assert_success
from tests.support.helpers import is_element_in_viewport


def element_send_keys(session, element, text):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/value".format(
            session_id=session.session_id,
            element_id=element.id),
        {"text": text})


def test_element_outside_of_not_scrollable_viewport(session, inline):
    session.url = inline("<input style=\"position: relative; left: -9999px;\">")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    assert_success(response)

    assert not is_element_in_viewport(session, element)


def test_element_outside_of_scrollable_viewport(session, inline):
    session.url = inline("<input style=\"margin-top: 102vh;\">")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    assert_success(response)

    assert is_element_in_viewport(session, element)


def test_contenteditable_element_outside_of_scrollable_viewport(session, inline):
    session.url = inline("<div contenteditable style=\"margin-top: 102vh;\"></div>")
    element = session.find.css("div", all=False)

    response = element_send_keys(session, element, "foo")
    assert_success(response)

    assert is_element_in_viewport(session, element)
