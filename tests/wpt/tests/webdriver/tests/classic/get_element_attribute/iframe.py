import pytest

from tests.support.classic.asserts import assert_success


def get_element_attribute(session, element_id, attr):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/attribute/{attr}".format(
            session_id=session.session_id,
            element_id=element_id,
            attr=attr))


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
@pytest.mark.parametrize("attr,expected", [
    ("value", "v"),
    ("disabled", "true")])
def test_element_attribute_in_frame(session, inline, iframe, attr, expected, domain):
    session.url = inline(iframe("<input id='x' value='v' disabled>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css("#x", all=False)
    result = get_element_attribute(session, element.id, attr)
    assert_success(result, expected)
