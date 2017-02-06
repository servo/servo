import webdriver

def dismiss_user_prompts(session):
    """Dismisses any open user prompts in windows."""
    current_window = session.window_handle

    for window in _windows(session):
        session.window_handle = window
        try:
            session.alert.dismiss()
        except webdriver.NoSuchAlertException:
            pass

    session.window_handle = current_window

def restore_windows(session):
    """Closes superfluous windows opened by the test without ending
    the session implicitly by closing the last window.
    """
    current_window = session.window_handle

    for window in _windows(session, exclude=[current_window]):
        session.window_handle = window
        if len(session.window_handles) > 1:
            session.close()

    session.window_handle = current_window

def switch_to_top_level_browsing_context(session):
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
