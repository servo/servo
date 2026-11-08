import pytest

from tests.support.classic.asserts import assert_success


def get_computed_label(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/computedlabel".format(
            session_id=session.session_id,
            element_id=element_id))

@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
@pytest.mark.parametrize("html,tag,expected", [
    ("<button>ok</button>", "button", "ok"),
    ("<button aria-label=foo>bar</button>", "button", "foo"),
    ("<button aria-labelledby=\"one two\"></button><div id=one>ok</div><div id=two>go</div>", "button", "ok go"),
    ("<label><input> foo</label>", "input", "foo")])
def test_computed_label_in_frame(session, inline, iframe, html, tag, expected, domain):
    session.url = inline(iframe(html, domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css(tag, all=False)
    result = get_computed_label(session, element.id)
    assert_success(result, expected)
