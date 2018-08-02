from tests.support.asserts import assert_error, assert_success


def fullscreen(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/fullscreen".format(**vars(session)))


def is_fullscreen(session):
    # At the time of writing, WebKit does not conform to the
    # Fullscreen API specification.
    #
    # Remove the prefixed fallback when
    # https://bugs.webkit.org/show_bug.cgi?id=158125 is fixed.
    return session.execute_script("""
        return !!(window.fullScreen || document.webkitIsFullScreen)
        """)


def test_no_browsing_context(session, create_window):
    session.window_handle = create_window()
    session.close()
    response = fullscreen(session)
    assert_error(response, "no such window")


def test_fullscreen(session):
    response = fullscreen(session)
    assert_success(response)

    assert is_fullscreen(session) is True


def test_payload(session):
    response = fullscreen(session)

    assert response.status == 200
    assert isinstance(response.body["value"], dict)

    value = response.body["value"]
    assert "width" in value
    assert "height" in value
    assert "x" in value
    assert "y" in value
    assert isinstance(value["width"], int)
    assert isinstance(value["height"], int)
    assert isinstance(value["x"], int)
    assert isinstance(value["y"], int)


def test_fullscreen_twice_is_idempotent(session):
    assert is_fullscreen(session) is False

    first_response = fullscreen(session)
    assert_success(first_response)
    assert is_fullscreen(session) is True

    second_response = fullscreen(session)
    assert_success(second_response)
    assert is_fullscreen(session) is True
