import pytest

from tests.support.classic.asserts import assert_success


def get_shadow_root(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/shadow".format(
            session_id=session.session_id,
            element_id=element_id))


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_get_element_shadow_root_in_frame(session, inline, iframe, domain):
    # Declarative shadow DOM builds the host entirely inside the frame.
    session.url = inline(iframe(
        "<div id='host'><template shadowrootmode='open'>"
        "<input id='in-shadow'></template></div>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    host = session.find.css("#host", all=False)
    response = get_shadow_root(session, host.id)
    value = assert_success(response)

    # A shadow-root reference must come back for the host in the frame.
    assert isinstance(value, dict)
    assert "shadow-6066-11e4-a52e-4f735466cecf" in value
