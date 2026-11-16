import pytest

from tests.support.classic.asserts import assert_success
from tests.support.classic.asserts import assert_element_has_focus


def element_send_keys(session, element, text):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/value".format(
            session_id=session.session_id,
            element_id=element.id),
        {"text": text})


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_send_keys_to_element_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<input id='x'>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css("#x", all=False)

    response = element_send_keys(session, element, "a")
    assert assert_success(response) is None
    assert session.execute_script("return document.querySelector('#x').value") == "a"

    assert_element_has_focus(element)
