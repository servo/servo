import pytest
from webdriver.error import NoSuchAlertException

from tests.support.sync import Poll


@pytest.fixture
def add_event_listeners(session):
    """Register listeners for tracked events on element."""
    def add_event_listeners(element, tracked_events):
        element.session.execute_script("""
            let element = arguments[0];
            let trackedEvents = arguments[1];

            if (!("events" in window)) {
              window.events = [];
            }

            for (var i = 0; i < trackedEvents.length; i++) {
              element.addEventListener(trackedEvents[i], function (event) {
                window.events.push(event.type);
              });
            }
            """, args=(element, tracked_events))
    return add_event_listeners


@pytest.fixture
def closed_frame(session, url):
    """Create a frame and remove it after switching to it.

    The removed frame will be kept selected, which allows to test for invalid
    browsing context references.
    """
    original_handle = session.window_handle
    new_handle = session.new_window()

    session.window_handle = new_handle

    session.url = url("/webdriver/tests/support/html/frames.html")

    subframe = session.find.css("#sub-frame", all=False)
    session.switch_frame(subframe)

    deleteframe = session.find.css("#delete-frame", all=False)
    session.switch_frame(deleteframe)

    button = session.find.css("#remove-parent", all=False)
    button.click()

    yield

    session.window.close()
    assert new_handle not in session.handles, "Unable to close window {}".format(new_handle)

    session.window_handle = original_handle


@pytest.fixture
def closed_window(session, inline):
    """Create a window and close it immediately.

    The window handle will be kept selected, which allows to test for invalid
    top-level browsing context references.
    """
    original_handle = session.window_handle
    new_handle = session.new_window()

    session.window_handle = new_handle
    session.url = inline("<input id='a' value='b'>")
    element = session.find.css("input", all=False)

    session.window.close()
    assert new_handle not in session.handles, "Unable to close window {}".format(new_handle)

    yield (original_handle, element)

    session.window_handle = original_handle


@pytest.fixture
def create_cookie(session, url):
    """Create a cookie."""
    def create_cookie(name, value, **kwargs):
        if kwargs.get("path", None) is not None:
            session.url = url(kwargs["path"])

        session.set_cookie(name, value, **kwargs)
        return session.cookies(name)

    return create_cookie


@pytest.fixture
def create_dialog(session):
    """Create a dialog (one of "alert", "prompt", or "confirm").

    Also it provides a function to validate that the dialog has been "handled"
    (either accepted or dismissed) by returning some value.
    """
    def create_dialog(dialog_type, text=None):
        assert dialog_type in ("alert", "confirm", "prompt"), (
            "Invalid dialog type: '%s'" % dialog_type)

        if text is None:
            text = ""

        assert isinstance(text, str), "`text` parameter must be a string"

        # Script completes itself when the user prompt has been opened.
        # For prompt() dialogs, add a value for the 'default' argument,
        # as some user agents (IE, for example) do not produce consistent
        # values for the default.
        session.execute_async_script("""
            let dialog_type = arguments[0];
            let text = arguments[1];

            setTimeout(function() {
              if (dialog_type == 'prompt') {
                window.dialog_return_value = window[dialog_type](text, '');
              } else {
                window.dialog_return_value = window[dialog_type](text);
              }
            }, 0);
            """, args=(dialog_type, text))

        wait = Poll(
            session,
            timeout=15,
            ignored_exceptions=NoSuchAlertException,
            message="No user prompt with text '{}' detected".format(text))
        wait.until(lambda s: s.alert.text == text)

    return create_dialog


@pytest.fixture
def create_frame(session):
    """Create an `iframe` element.

    The element will be inserted into the document of the current browsing
    context. Return a reference to the newly-created element.
    """
    def create_frame():
        append = """
            var frame = document.createElement('iframe');
            document.body.appendChild(frame);
            return frame;
        """
        return session.execute_script(append)

    return create_frame


@pytest.fixture
def stale_element(current_session, get_test_page):
    """Create a stale element reference

    The document will be loaded in the top-level or child browsing context.
    Before the requested element or its shadow root is returned the element
    is removed from the document's DOM.
    """
    def stale_element(css_value, as_frame=False, want_shadow_root=False):
        current_session.url = get_test_page(as_frame=as_frame)

        if as_frame:
            frame = current_session.find.css("iframe", all=False)
            current_session.switch_frame(frame)

        element = current_session.find.css(css_value, all=False)
        shadow_root = element.shadow_root if want_shadow_root else None

        current_session.execute_script("arguments[0].remove();", args=[element])

        return shadow_root if want_shadow_root else element

    return stale_element


@pytest.fixture
def get_test_page(iframe, inline):
    def get_test_page(
        as_frame=False,
        frame_doc=None,
        shadow_doc=None,
        nested_shadow_dom=False
    ):
        if frame_doc is None:
            frame_doc = """<div id="in-frame"><input type="checkbox"/></div>"""

        if shadow_doc is None:
            shadow_doc = """
                <div id="in-shadow-dom">
                    <input type="checkbox"/>
                </div>
            """

        definition_inner_shadow_dom = ""
        if nested_shadow_dom:
            definition_inner_shadow_dom = f"""
                customElements.define('inner-custom-element',
                    class extends HTMLElement {{
                        constructor() {{
                            super();
                            this.attachShadow({{mode: "open"}}).innerHTML = `
                                {shadow_doc}
                            `;
                        }}
                    }}
                );
            """
            shadow_doc = """
                <style>
                    inner-custom-element {
                        display:block; width:20px; height:20px;
                    }
                </style>
                <div id="in-nested-shadow-dom">
                    <inner-custom-element></inner-custom-element>
                </div>
                """

        page_data = f"""
            <style>
                custom-element {{
                    display:block; width:20px; height:20px;
                }}
            </style>
            <div id="with-children"><p><span></span></p><br/></div>
            <div id="with-text-node">Lorem</div>
            <div id="with-comment"><!-- Comment --></div>

            <input id="button" type="button"/>
            <input id="checkbox" type="checkbox"/>
            <input id="file" type="file"/>
            <input id="hidden" type="hidden"/>
            <input id="text" type="text"/>

            {iframe(frame_doc)}

            <svg></svg>

            <custom-element id="custom-element"></custom-element>
            <script>
                var svg = document.querySelector("svg");
                svg.setAttributeNS("http://www.w3.org/2000/svg", "svg:foo", "bar");

                customElements.define("custom-element",
                    class extends HTMLElement {{
                        constructor() {{
                            super();
                            this.attachShadow({{mode: "open"}}).innerHTML = `
                                {shadow_doc}
                            `;
                        }}
                    }}
                );
                {definition_inner_shadow_dom}
            </script>"""

        if as_frame:
            return inline(iframe(page_data))
        else:
            return inline(page_data)

    return get_test_page
