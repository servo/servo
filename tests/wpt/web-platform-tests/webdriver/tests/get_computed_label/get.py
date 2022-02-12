import pytest

from webdriver.error import NoSuchAlertException

from tests.support.asserts import assert_error, assert_success


def get_computed_label(session, element):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/computedlabel".format(
            session_id=session.session_id,
            element_id=element))


def test_no_browsing_context(session, closed_window):
    response = get_computed_label(session)
    assert_error(response, "no such window")


def test_no_user_prompt(session):
    response = get_computed_label(session)
    assert_error(response, "no such alert")


@pytest.mark.parametrize("html,tag,label", [
    ("<button>ok</button>", "button", "ok"),
    ("<button aria-labelledby=\"one two\"></button><div id=one>ok</div><div id=two>go</div>", "button", "ok go"),
    ("<button aria-label=foo>bar</button>", "button", "foo"),
    ("<label><input> foo</label>", "input", "foo"),
    ("<label for=b>foo<label><input id=b>", "input", "foo")])
def test_get_computed_label(session, inline, html, tag, label):
    session.url = inline("{0}".format(tag))
    element = session.find.css(tag, all=False)
    result = get_computed_label(session, element.id)
    assert_success(result, label)
