import pytest
from tests.support.fixtures import (
    configuration, create_dialog, create_frame, create_window, http,
    new_session, server_config, session, url)

pytest.fixture(scope="session")(configuration)
pytest.fixture()(create_dialog)
pytest.fixture()(create_frame)
pytest.fixture()(create_window)
pytest.fixture()(http)
pytest.fixture(scope="function")(new_session)
pytest.fixture()(server_config)
pytest.fixture(scope="function")(session)
pytest.fixture()(url)
