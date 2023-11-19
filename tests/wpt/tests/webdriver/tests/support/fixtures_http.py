import base64

import pytest
from webdriver.error import NoSuchAlertException

from tests.support.image import png_dimensions, ImageDifference
from tests.support.sync import Poll


@pytest.fixture
def add_event_listeners():
    """Register listeners for tracked events on element."""
    def add_event_listeners(element, tracked_events):
        element.session.execute_script("""
            const element = arguments[0];
            const trackedEvents = arguments[1];

            if (!("events" in window)) {
              window.events = [];
            }

            for (let i = 0; i < trackedEvents.length; i++) {
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
def load_pdf_http(current_session, test_page_with_pdf_js):
    """Load a PDF document in the browser using pdf.js"""
    def load_pdf_http(encoded_pdf_data):
        current_session.url = test_page_with_pdf_js(encoded_pdf_data)

    return load_pdf_http


@pytest.fixture
def render_pdf_to_png_http(current_session, url):
    """Render a PDF document to png"""

    def render_pdf_to_png_http(
        encoded_pdf_data, page=1
    ):
        current_session.url = url(path="/print_pdf_runner.html")
        result = current_session.execute_async_script(f"""arguments[0](window.render("{encoded_pdf_data}"))""")
        index = page - 1

        assert 0 <= index < len(result)

        image_string = result[index]
        image_string_without_data_type = image_string[image_string.find(",") + 1:]

        return base64.b64decode(image_string_without_data_type)

    return render_pdf_to_png_http


@pytest.fixture
def compare_png_http(current_session, url):
    def compare_png_http(img1, img2):
        """Calculate difference statistics between two PNG images.

        :param img1: Bytes of first PNG image
        :param img2: Bytes of second PNG image
        :returns: ImageDifference representing the total number of different pixels,
                and maximum per-channel difference between the images.
        """
        if img1 == img2:
            return ImageDifference(0, 0)

        width, height = png_dimensions(img1)
        assert (width, height) == png_dimensions(img2)

        current_session.url = url("/webdriver/tests/support/html/render.html")
        result = current_session.execute_async_script(
            "const callback = arguments[arguments.length - 1]; callback(compare(arguments[0], arguments[1], arguments[2], arguments[3]))",
            args=[base64.encodebytes(img1).decode(), base64.encodebytes(img2).decode(), width, height],
        )

        assert "maxDifference" in result
        assert "totalPixels" in result

        return ImageDifference(result["totalPixels"], result["maxDifference"])

    return compare_png_http
