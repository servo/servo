import pytest

from webdriver.error import NoSuchWindowException


@pytest.fixture(name="session")
def fixture_session(capabilities, session):
    """Prevent re-using existent history by running the test in a new window."""
    original_handle = session.window_handle
    session.window_handle = session.new_window()

    yield session

    try:
        session.window.close()
    except NoSuchWindowException:
        pass

    session.window_handle = original_handle
