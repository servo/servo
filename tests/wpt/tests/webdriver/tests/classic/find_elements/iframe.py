import pytest

from tests.support.classic.asserts import assert_success


def find_elements(session, using, value):
    return session.transport.send(
        "POST", "session/{session_id}/elements".format(**vars(session)),
        {"using": using, "value": value})


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_find_elements_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<p class=x>a</p><p class=x>b</p>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    response = find_elements(session, "css selector", ".x")
    value = assert_success(response)

    assert isinstance(value, list)
    assert len(value) == 2
