import pytest
import time

from webdriver.error import NoSuchWindowException


@pytest.fixture
def session_new_window(capabilities, session):
    # Prevent unreleased dragged elements by running the test in a new window.
    original_handle = session.window_handle
    session.window_handle = session.new_window()

    yield session

    try:
        session.window.close()
    except NoSuchWindowException:
        pass

    session.window_handle = original_handle


@pytest.fixture
def key_chain(session):
    return session.actions.sequence("key", "keyboard_id")


@pytest.fixture
def mouse_chain(session):
    return session.actions.sequence(
        "pointer",
        "pointer_id",
        {"pointerType": "mouse"})


@pytest.fixture
def touch_chain(session):
    return session.actions.sequence(
        "pointer",
        "pointer_id",
        {"pointerType": "touch"})


@pytest.fixture
def pen_chain(session):
    return session.actions.sequence(
        "pointer",
        "pointer_id",
        {"pointerType": "pen"})


@pytest.fixture
def none_chain(session):
    return session.actions.sequence("none", "none_id")


@pytest.fixture
def wheel_chain(session):
    return session.actions.sequence("wheel", "wheel_id")


@pytest.fixture(autouse=True)
def release_actions(session):
    # release all actions after each test
    # equivalent to a teardown_function, but with access to session fixture
    yield

    # Avoid frequent intermittent failures on Mozilla Windows CI with
    # perform_actions tests:
    # https://bugzilla.mozilla.org/show_bug.cgi?id=1879556
    if (
        session.capabilities["browserName"] == "firefox"
        and session.capabilities["platformName"] == "windows"
    ):
        time.sleep(0.1)

    try:
        session.actions.release()
    except NoSuchWindowException:
        pass


@pytest.fixture
def key_reporter(session, test_actions_page, request):
    """Represents focused input element from `test_actions_page` fixture."""
    input_el = session.find.css("#keys", all=False)
    input_el.click()
    session.execute_script("resetEvents();")
    return input_el


@pytest.fixture
def test_actions_page(session, url):
    session.url = url("/webdriver/tests/support/html/test_actions.html")


@pytest.fixture
def test_actions_scroll_page(session, url):
    session.url = url("/webdriver/tests/support/html/test_actions_scroll.html")


@pytest.fixture
def test_actions_pointer_page(session, url):
    session.url = url("/webdriver/tests/support/html/test_actions_pointer.html")
