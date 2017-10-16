# META: timeout=long

import json


_window_id = "window-fcc6-11e5-b4f8-330a88ab9d7f"
_frame_id = "frame-075b-4da1-b6ba-e579c2d3230a"

def test_initial_window(session):
    # non-auxiliary top-level browsing context
    raw_json = session.execute_script("return window;")
    obj = json.loads(raw_json)
    assert len(obj) == 1
    assert _window_id in obj
    handle = obj[_window_id]
    assert handle in session.window_handles


def test_window_open(session):
    # auxiliary browsing context
    session.execute_script("window.foo = window.open()")
    raw_json = session.execute_script("return window.foo;")
    obj = json.loads(raw_json)
    assert len(obj) == 1
    assert _window_id in obj
    handle = obj[_window_id]
    assert handle in session.window_handles


def test_frame(session):
    # nested browsing context
    append = """
        window.frame = document.createElement('iframe');
        document.body.appendChild(frame);
    """
    session.execute_script(append)
    raw_json = session.execute_script("return frame.contentWindow;")
    obj = json.loads(raw_json)
    assert len(obj) == 1
    assert _frame_id in obj
    handle = obj[_frame_id]
    assert handle not in session.window_handles
