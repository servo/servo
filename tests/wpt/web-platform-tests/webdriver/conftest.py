import pytest
from support.fixtures import (
    create_frame, create_session, create_window, http, server_config, session,
    url)

pytest.fixture()(create_frame)
pytest.fixture()(create_session)
pytest.fixture()(create_window)
pytest.fixture()(http)
pytest.fixture()(server_config)
pytest.fixture(scope="function")(session)
pytest.fixture()(url)
