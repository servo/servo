import pytest

from tests.support.classic.asserts import assert_success


def element_clear(session, element):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/clear".format(
            session_id=session.session_id,
            element_id=element.id))


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_clear_element_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<input id='x' value='preset'>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css("#x", all=False)
    response = element_clear(session, element)
    assert assert_success(response) is None

    # The clear's effect must be observable in the same frame.
    assert session.execute_script("return document.querySelector('#x').value") == ""
