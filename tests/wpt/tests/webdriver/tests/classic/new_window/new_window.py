from tests.support.asserts import assert_success

from . import opener, window_name


def new_window(session, type_hint=None):
    return session.transport.send(
        "POST", "session/{session_id}/window/new".format(**vars(session)),
        {"type": type_hint})


def test_payload(session):
    original_handles = session.handles

    response = new_window(session, type_hint="window")
    value = assert_success(response)
    handles = session.handles
    assert len(handles) == len(original_handles) + 1
    assert value["handle"] in handles
    assert value["handle"] not in original_handles

    # On Android applications have a single window only and a new tab will
    # be opened instead.
    if session.capabilities["platformName"] == "android":
        assert value["type"] == "tab"
    else:
        assert value["type"] == "window"


def test_keeps_current_window_handle(session):
    original_handle = session.window_handle

    response = new_window(session, type_hint="window")
    value = assert_success(response)

    assert session.window_handle == original_handle


def test_opens_about_blank_in_new_window(session, inline):
    url = inline("<p>foo")
    session.url = url

    response = new_window(session, type_hint="window")
    value = assert_success(response)

    assert session.url == url

    session.window_handle = value["handle"]
    assert session.url == "about:blank"


def test_sets_no_window_name(session):
    response = new_window(session, type_hint="window")
    value = assert_success(response)

    session.window_handle = value["handle"]
    assert window_name(session) == ""


def test_sets_no_opener(session):
    response = new_window(session, type_hint="window")
    value = assert_success(response)

    session.window_handle = value["handle"]
    assert opener(session) is None


def test_focus_content(session, inline):
    response = new_window(session, type_hint="window")
    value = assert_success(response)

    session.window_handle = value["handle"]

    session.url = inline("""
        <span contenteditable="true"> abc </span>
        <script>
            const selection = getSelection();
            window.onload = async() => {
                const initial = document.querySelector("span");
                initial.focus();
                initial.setAttribute(
                    "_focused",
                    selection.anchorNode == initial.firstChild
                );
            }
        </script>
    """)

    elem = session.find.css("span", all=False)
    assert elem.attribute("_focused") == "true"
