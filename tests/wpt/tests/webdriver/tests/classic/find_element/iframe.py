import pytest

from tests.support.classic.asserts import assert_success


def find_element(session, using, value):
    return session.transport.send(
        "POST", "session/{session_id}/element".format(**vars(session)),
        {"using": using, "value": value})


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_find_element_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<div id='target'>bar</div>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    response = find_element(session, "css selector", "#target")
    value = assert_success(response)

    assert session.execute_script("return arguments[0].id", args=[value]) == "target"
