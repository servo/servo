import base64
import copy

import pytest
import pytest_asyncio
import webdriver
from webdriver.error import NoSuchAlertException, NoSuchWindowException

import tests.support.fixtures as global_fixtures
from tests.support import defaults
from tests.support.classic.helpers import cleanup_session
from tests.support.helpers import deep_update
from tests.support.image import png_dimensions, ImageDifference
from tests.support.sync import Poll


SCRIPT_TIMEOUT = 1
PAGE_LOAD_TIMEOUT = 3
IMPLICIT_WAIT_TIMEOUT = 0


@pytest_asyncio.fixture(scope="function")
async def session(capabilities, configuration):
    """Create and start a session for a test that does not itself test session creation.

    By default the session will stay open after each test, but we always try to start a
    new one and assume that if that fails there is already a valid session. This makes it
    possible to recover from some errors that might leave the session in a bad state, but
    does not demand that we start a new session per test.
    """
    # Update configuration capabilities with custom ones from the
    # capabilities fixture, which can be set by tests
    caps = copy.deepcopy(configuration["capabilities"])
    deep_update(caps, capabilities)
    caps = {"alwaysMatch": caps}

    await global_fixtures.reset_current_session_if_necessary(caps)

    if global_fixtures.get_current_session() is None:
        global_fixtures.set_current_session(webdriver.Session(
            configuration["host"],
            configuration["port"],
            capabilities=caps))

    try:
        session = global_fixtures.get_current_session()
        session.start()

        # Enforce a fixed default window size and position
        if session.capabilities.get("setWindowRect"):
            session.window.size = defaults.WINDOW_SIZE
            session.window.position = defaults.WINDOW_POSITION

        # Set default timeouts
        multiplier = configuration["timeout_multiplier"]
        session.timeouts.implicit = IMPLICIT_WAIT_TIMEOUT * multiplier
        session.timeouts.page_load = PAGE_LOAD_TIMEOUT * multiplier
        session.timeouts.script = SCRIPT_TIMEOUT * multiplier

        yield session

        cleanup_session(session)

    except Exception:
        # Make sure we end up in a known state if something goes wrong.
        global_fixtures.get_current_session().end()
        raise


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
    session.switch_to_frame(subframe)

    deleteframe = session.find.css("#delete-frame", all=False)
    session.switch_to_frame(deleteframe)

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

        def check_alert_text(s):
            assert s.alert.text == text, f"No user prompt with text '{text}' detected"

        wait = Poll(session, timeout=15,
                    ignored_exceptions=NoSuchAlertException)
        wait.until(check_alert_text)

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
def new_tab_classic(session):
    """Create a new tab to run the test isolated."""
    original_handle = session.window_handle
    new_handle = session.new_window(type_hint="tab")

    session.window_handle = new_handle

    yield

    try:
        # Make sure to close the correct tab that we opened before.
        session.window_handle = new_handle
        session.window.close()
    except NoSuchWindowException:
        pass

    session.window_handle = original_handle


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
            current_session.switch_to_frame(frame)

        element = current_session.find.css(css_value, all=False)
        shadow_root = element.shadow_root if want_shadow_root else None

        current_session.execute_script("arguments[0].remove();", args=[element])

        return shadow_root if want_shadow_root else element

    return stale_element


@pytest.fixture
def load_pdf_classic(current_session, test_page_with_pdf_js):
    """Load a PDF document in the browser using pdf.js"""
    def load_pdf_classic(encoded_pdf_data):
        current_session.url = test_page_with_pdf_js(encoded_pdf_data)

    return load_pdf_classic


@pytest.fixture
def render_pdf_to_png_classic(current_session, url):
    """Render a PDF document to png"""

    def render_pdf_to_png_classic(encoded_pdf_data, page=1):
        current_session.url = url(path="/print_pdf_runner.html")
        result = current_session.execute_async_script(f"""arguments[0](window.render("{encoded_pdf_data}"))""")
        index = page - 1

        assert 0 <= index < len(result)

        image_string = result[index]
        image_string_without_data_type = image_string[image_string.find(",") + 1:]

        return base64.b64decode(image_string_without_data_type)

    return render_pdf_to_png_classic


@pytest.fixture
def compare_png_classic(current_session, url):
    def compare_png_classic(img1, img2):
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

    return compare_png_classic


@pytest.fixture()
def available_screen_size(session):
    """Return the effective available screen size (width/height).

    This is size which excludes any fixed window manager elements like menu
    bars, and the dock on MacOS.
    """
    return tuple(
        session.execute_script(
            """
        return [
            screen.availWidth,
            screen.availHeight,
        ];
        """
        )
    )


@pytest.fixture()
def minimal_screen_position(session):
    """Return the minimal position (x/y) a window can be positioned at."""
    return tuple(
        session.execute_script(
            """
        return [
            screen.availLeft,
            screen.availTop,
        ];
        """
        )
    )


@pytest.fixture()
def screen_size(session):
    """Return the size (width/height) of the screen."""
    return tuple(
        session.execute_script(
            """
        return [
            screen.width,
            screen.height,
        ];
        """
        )
    )
