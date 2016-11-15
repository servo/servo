import json
import pytest
import webdriver


@pytest.mark.xfail(raises=webdriver.UnsupportedOperationException)
def test_window_size_types(http, session):
    session.start()
    with http.get("/session/%s/window/size" % session.session_id) as resp:
        assert resp.status == 200
        body = json.load(resp)
    assert "width" in body
    assert "height" in body
    assert isinstance(body["width"], int)
    assert isinstance(body["height"], int)

    size = session.window.size
    assert isinstance(size, tuple)
    assert isinstance(size[0], int)
    assert isinstance(size[1], int)


@pytest.mark.xfail(raises=webdriver.UnsupportedOperationException)
def test_window_resize(session):
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

@pytest.mark.xfail(raises=webdriver.UnsupportedOperationException)
def test_window_position_types(http, session):
    session.start()
    with http.get("/session/%s/window/position" % session.session_id) as resp:
        assert resp.status == 200
        body = json.load(resp)
    assert "x" in body
    assert "y" in body
    assert isinstance(body["x"], int)
    assert isinstance(body["y"], int)

    size = session.window.position
    assert isinstance(size, tuple)
    assert isinstance(size[0], int)
    assert isinstance(size[1], int)
