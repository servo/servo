import json
import pytest
import webdriver


def window_size_supported(session):
    try:
        session.window.size = ("a", "b")
    except webdriver.UnsupportedOperationException:
        return False
    except webdriver.InvalidArgumentException:
        return True

def window_position_supported(session):
    try:
        session.window.position = ("a", "b")
    except webdriver.UnsupportedOperationException:
        return False
    except webdriver.InvalidArgumentException:
        return True


def test_window_resize(session):
    if not window_size_supported(session):
        pytest.skip()

    # setting the window size by webdriver is synchronous
    # so we should see the results immediately

    session.window.size = (400, 500)
    assert session.window.size == (400, 500)

    session.window.size = (500, 600)
    assert session.window.size == (500, 600)


"""
TODO(ato):

    Disable test because the while statements are wrong.
    To fix this properly we need to write an explicit wait utility.

def test_window_resize_by_script(session):
    # setting the window size by JS is asynchronous
    # so we poll waiting for the results

    size0 = session.window.size

    session.execute_script("window.resizeTo(700, 800)")
    size1 = session.window.size
    while size0 == size1:
        size1 = session.window.size
    assert size1 == (700, 800)

    session.execute_script("window.resizeTo(800, 900)")
    size2 = session.window.size
    while size1 == size2:
        size2 = session.window.size
        assert size2 == (800, 900)
    assert size2 == {"width": 200, "height": 100}
"""
