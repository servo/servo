import pytest

from tests.support.classic.asserts import assert_success


def find_elements(session, shadow_id, using, value):
    return session.transport.send(
        "POST", "session/{session_id}/shadow/{shadow_id}/elements".format(
            session_id=session.session_id,
            shadow_id=shadow_id),
        {"using": using, "value": value})


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_find_elements_from_shadow_root_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe(
        "<div id='host'><template shadowrootmode='open'>"
        "<input class=s><input class=s></template></div>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    host = session.find.css("#host", all=False)
    shadow = host.shadow_root

    response = find_elements(session, shadow.id, "css selector", ".s")
    value = assert_success(response)

    assert isinstance(value, list)
    assert len(value) == 2
