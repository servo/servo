import pytest

from tests.support.classic.asserts import assert_success


def get_computed_role(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/computedrole".format(
            session_id=session.session_id,
            element_id=element_id))

@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
@pytest.mark.parametrize("html,tag,expected", [
    ("<article>foo</article>", "article", "article"),
    ("<input role=searchbox>", "input", "searchbox"),
    ("<img role=button tabindex=0>", "img", "button")])
def test_computed_role_in_frame(session, inline, iframe, html, tag, expected, domain):
    session.url = inline(iframe(html, domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css(tag, all=False)
    result = get_computed_role(session, element.id)
    assert_success(result, expected)