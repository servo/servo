from tests.support.asserts import assert_error, assert_success


def get_window_handle(session):
    return session.transport.send(
        "GET", "session/{session_id}/window".format(**vars(session)))


def test_no_top_browsing_context(session, closed_window):
    response = get_window_handle(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = get_window_handle(session)
    assert_success(response, session.window_handle)


def test_basic(session):
    response = get_window_handle(session)
    assert_success(response, session.window_handle)


def test_navigation_with_coop_headers(session, url):
    base_path = ("/webdriver/tests/support/html/subframe.html" +
                 "?pipe=header(Cross-Origin-Opener-Policy,same-origin)")

    session.url = url(base_path, protocol="https")
    response = get_window_handle(session)
    first_handle = assert_success(response)

    # navigating to another domain with COOP headers will force a process change
    # in most browsers
    session.url = url(base_path, protocol="https", domain="alt")
    response = get_window_handle(session)
    second_handle = assert_success(response)

    assert first_handle == second_handle
