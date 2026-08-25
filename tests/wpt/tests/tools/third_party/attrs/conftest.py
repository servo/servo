# SPDX-License-Identifier: MIT

import pytest

from hypothesis import HealthCheck, settings

from attr._compat import PY310


@pytest.fixture(name="slots", params=(True, False))
def _slots(request):
    return request.param


@pytest.fixture(name="frozen", params=(True, False))
def _frozen(request):
    return request.param


def pytest_configure(config):
    # HealthCheck.too_slow causes more trouble than good -- especially in CIs.
    settings.register_profile(
        "patience", settings(suppress_health_check=[HealthCheck.too_slow])
    )
    settings.load_profile("patience")


collect_ignore = []
if not PY310:
    collect_ignore.extend(["tests/test_pattern_matching.py"])
