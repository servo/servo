import pytest

from tests.support.classic.asserts import assert_same_element, assert_success


def get_active_element(session):
    return session.transport.send(
        "GET", "session/{session_id}/element/active".format(**vars(session)))


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_get_active_element_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<input id='in'>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    # Neither autofocus nor focus() can focus an element in a cross-origin frame
    # without user activation ("focus-without-user-activation" defaults to 'self'),
    # so focus the input by clicking it.
    expected = session.find.css("#in", all=False)
    expected.click()

    response = get_active_element(session)
    value = assert_success(response)

    # The returned reference must be the focused input inside the frame.
    assert_same_element(session, value, expected)
