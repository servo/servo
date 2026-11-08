import pytest

from tests.support.classic.asserts import assert_success


def get_window_handles(session):
    return session.transport.send(
        "GET", "session/{session_id}/window/handles".format(**vars(session)))


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_window_with_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<p id=child>foo</p>", domain=domain))

    # Make sure that the frame has loaded. A browser may put a cross-site frame in a separate process,
    # which must not make the window appear twice.
    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)
    session.find.css("#child", all=False)

    response = get_window_handles(session)
    assert_success(response, [session.window_handle])


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_window_after_navigating_away_from_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<p id=child>foo</p>", domain=domain))
    session.url = inline("<p>bar</p>")

    response = get_window_handles(session)
    assert_success(response, [session.window_handle])
