import pytest

from tests.support.classic.asserts import assert_success


def get_element_text(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/text".format(
            session_id=session.session_id,
            element_id=element_id))


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_element_text_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<p id='x'>hello</p>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css("#x", all=False)
    result = get_element_text(session, element.id)
    assert_success(result, "hello")
