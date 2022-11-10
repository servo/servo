# SPDX-License-Identifier: MIT

from __future__ import absolute_import, division, print_function

from hypothesis import HealthCheck, settings

from attr._compat import PY36, PY310


def pytest_configure(config):
    # HealthCheck.too_slow causes more trouble than good -- especially in CIs.
    settings.register_profile(
        "patience", settings(suppress_health_check=[HealthCheck.too_slow])
    )
    settings.load_profile("patience")


collect_ignore = []
if not PY36:
    collect_ignore.extend(
        [
            "tests/test_annotations.py",
            "tests/test_hooks.py",
            "tests/test_init_subclass.py",
            "tests/test_next_gen.py",
        ]
    )
if not PY310:
    collect_ignore.extend(["tests/test_pattern_matching.py"])
