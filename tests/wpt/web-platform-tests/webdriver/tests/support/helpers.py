from __future__ import print_function

import math
import sys

import webdriver

from tests.support import defaults
from tests.support.sync import Poll


def ignore_exceptions(f):
    def inner(*args, **kwargs):
        try:
            return f(*args, **kwargs)
        except webdriver.error.WebDriverException as e:
            print("Ignored exception %s" % e, file=sys.stderr)
    inner.__name__ = f.__name__
    return inner


def cleanup_session(session):
    """Clean-up the current session for a clean state."""
    @ignore_exceptions
    def _dismiss_user_prompts(session):
        """Dismiss any open user prompts in windows."""
        current_window = session.window_handle

        for window in _windows(session):
            session.window_handle = window
            try:
                session.alert.dismiss()
            except webdriver.NoSuchAlertException:
                pass

        session.window_handle = current_window

    @ignore_exceptions
    def _ensure_valid_window(session):
        """If current window was closed, ensure to have a valid one selected."""
        try:
            session.window_handle
        except webdriver.NoSuchWindowException:
            session.window_handle = session.handles[0]

    @ignore_exceptions
    def _restore_timeouts(session):
        """Restore modified timeouts to their default values."""
        session.timeouts.implicit = defaults.IMPLICIT_WAIT_TIMEOUT
        session.timeouts.page_load = defaults.PAGE_LOAD_TIMEOUT
        session.timeouts.script = defaults.SCRIPT_TIMEOUT

    @ignore_exceptions
    def _restore_window_state(session):
        """Reset window to an acceptable size.

        This also includes bringing it out of maximized, minimized,
        or fullscreened state.
        """
        session.window.size = defaults.WINDOW_SIZE

    @ignore_exceptions
    def _restore_windows(session):
        """Close superfluous windows opened by the test.

        It will not end the session implicitly by closing the last window.
        """
        current_window = session.window_handle

        for window in _windows(session, exclude=[current_window]):
            session.window_handle = window
            if len(session.handles) > 1:
                session.close()

        session.window_handle = current_window

    _restore_timeouts(session)
    _ensure_valid_window(session)
    _dismiss_user_prompts(session)
    _restore_windows(session)
    _restore_window_state(session)
    _switch_to_top_level_browsing_context(session)


@ignore_exceptions
def _switch_to_top_level_browsing_context(session):
    """If the current browsing context selected by WebDriver is a
    `<frame>` or an `<iframe>`, switch it back to the top-level
    browsing context.
    """
    session.switch_frame(None)


def _windows(session, exclude=None):
    """Set of window handles, filtered by an `exclude` list if
    provided.
    """
    if exclude is None:
        exclude = []
    wins = [w for w in session.handles if w not in exclude]
    return set(wins)


def clear_all_cookies(session):
    """Removes all cookies associated with the current active document"""
    session.transport.send("DELETE", "session/%s/cookie" % session.session_id)


def document_dimensions(session):
    return tuple(session.execute_script("""
        let rect = document.documentElement.getBoundingClientRect();
        return [rect.width, rect.height];
        """))


def center_point(element):
    """Calculates the in-view center point of a web element."""
    inner_width, inner_height = element.session.execute_script(
        "return [window.innerWidth, window.innerHeight]")
    rect = element.rect

    # calculate the intersection of the rect that is inside the viewport
    visible = {
        "left": max(0, min(rect["x"], rect["x"] + rect["width"])),
        "right": min(inner_width, max(rect["x"], rect["x"] + rect["width"])),
        "top": max(0, min(rect["y"], rect["y"] + rect["height"])),
        "bottom": min(inner_height, max(rect["y"], rect["y"] + rect["height"])),
    }

    # arrive at the centre point of the visible rectangle
    x = (visible["left"] + visible["right"]) / 2.0
    y = (visible["top"] + visible["bottom"]) / 2.0

    # convert to CSS pixels, as centre point can be float
    return (math.floor(x), math.floor(y))


def document_hidden(session):
    """Polls for the document to become hidden."""
    def hidden(session):
        return session.execute_script("return document.hidden")
    return Poll(session, timeout=3, raises=None).until(hidden)


def document_location(session):
    """
    Unlike ``webdriver.Session#url``, which always returns
    the top-level browsing context's URL, this returns
    the current browsing context's active document's URL.
    """
    return session.execute_script("return document.location.href")


def element_rect(session, element):
    return session.execute_script("""
        let element = arguments[0];
        let rect = element.getBoundingClientRect();

        return {
            x: rect.left + window.pageXOffset,
            y: rect.top + window.pageYOffset,
            width: rect.width,
            height: rect.height,
        };
        """, args=(element,))


def is_element_in_viewport(session, element):
    """Check if element is outside of the viewport"""
    return session.execute_script("""
        let el = arguments[0];

        let rect = el.getBoundingClientRect();
        let viewport = {
          height: window.innerHeight || document.documentElement.clientHeight,
          width: window.innerWidth || document.documentElement.clientWidth,
        };

        return !(rect.right < 0 || rect.bottom < 0 ||
            rect.left > viewport.width || rect.top > viewport.height)
    """, args=(element,))


def is_fullscreen(session):
    # At the time of writing, WebKit does not conform to the
    # Fullscreen API specification.
    #
    # Remove the prefixed fallback when
    # https://bugs.webkit.org/show_bug.cgi?id=158125 is fixed.
    return session.execute_script("""
        return !!(window.fullScreen || document.webkitIsFullScreen)
        """)


def document_dimensions(session):
    return tuple(session.execute_script("""
        let {devicePixelRatio} = window;
        let {width, height} = document.documentElement.getBoundingClientRect();
        return [width * devicePixelRatio, height * devicePixelRatio];
        """))


def screen_size(session):
    """Returns the available width/height size of the screen."""
    return tuple(session.execute_script("""
        return [
            screen.availWidth,
            screen.availHeight,
        ];
        """))


def available_screen_size(session):
    """
    Returns the effective available screen width/height size,
    excluding any fixed window manager elements.
    """
    return tuple(session.execute_script("""
        return [
            screen.availWidth - screen.availLeft,
            screen.availHeight - screen.availTop,
        ];
        """))
