import pytest

from tests.support.classic.asserts import assert_success


def find_element(session, element_id, using, value):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/element".format(
            session_id=session.session_id,
            element_id=element_id),
        {"using": using, "value": value})


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_find_element_from_element_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<div id='root'><span id='child'>x</span></div>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    root = session.find.css("#root", all=False)

    response = find_element(session, root.id, "css selector", "#child")
    value = assert_success(response)

    assert session.execute_script("return arguments[0].id", args=[value]) == "child"
