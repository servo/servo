from __future__ import print_function

import sys

import webdriver

from tests.support import defaults

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


def document_dimensions(session):
    return tuple(session.execute_script("""
        let {devicePixelRatio} = window;
        let {width, height} = document.documentElement.getBoundingClientRect();
        return [width * devicePixelRatio, height * devicePixelRatio];
        """))
