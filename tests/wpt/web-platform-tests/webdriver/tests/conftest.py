import pytest

from tests.support.fixtures import (
    add_event_listeners,
    configuration,
    closed_window,
    create_cookie,
    create_dialog,
    create_frame,
    create_window,
    current_session,
    http,
    server_config,
    session,
    url)

def pytest_configure(config):
    # register the capabilities marker
    config.addinivalue_line("markers",
        "capabilities: mark test to use capabilities")


@pytest.fixture
def capabilities():
    """Default capabilities to use for a new WebDriver session."""
    return {}


def pytest_generate_tests(metafunc):
    if "capabilities" in metafunc.fixturenames:
        marker = metafunc.definition.get_closest_marker(name="capabilities")
        if marker:
            metafunc.parametrize("capabilities", marker.args, ids=None)


pytest.fixture()(add_event_listeners)
pytest.fixture(scope="session")(configuration)
pytest.fixture()(create_cookie)
pytest.fixture()(create_dialog)
pytest.fixture()(create_frame)
pytest.fixture()(create_window)
pytest.fixture(scope="function")(current_session)
pytest.fixture()(http)
pytest.fixture()(server_config)
pytest.fixture(scope="function")(session)
pytest.fixture()(url)

# Fixtures for specific tests
pytest.fixture()(closed_window)
