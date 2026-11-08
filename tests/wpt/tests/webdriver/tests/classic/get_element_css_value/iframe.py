import pytest

from tests.support.classic.asserts import assert_success


def get_element_css_value(session, element_id, prop):
    return session.transport.send(
        "GET",
        "session/{session_id}/element/{element_id}/css/{prop}".format(
            session_id=session.session_id,
            element_id=element_id,
            prop=prop
        )
    )


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_element_css_value_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<div id='x' style='display:none'>foo</div>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css("#x", all=False)
    result = get_element_css_value(session, element.id, "display")
    assert_success(result, "none")
