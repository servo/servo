# -*- coding: utf-8 -*-
from hypothesis import settings, HealthCheck

import pytest
import helpers

# Set up a CI profile that allows slow example generation.
settings.register_profile(
    "travis",
    settings(suppress_health_check=[HealthCheck.too_slow])
)


@pytest.fixture
def frame_factory():
    return helpers.FrameFactory()
