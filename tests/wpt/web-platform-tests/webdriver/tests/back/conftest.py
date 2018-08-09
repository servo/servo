import pytest

from webdriver.error import NoSuchWindowException


@pytest.fixture(name="session")
def fixture_session(capabilities, session, create_window):
    """Prevent re-using existent history by running the test in a new window."""
    original_handle = session.window_handle
    session.window_handle = create_window()

    yield session

    try:
        session.close()
    except NoSuchWindowException:
        pass

    session.window_handle = original_handle
