import sys

import pytest
from tests.support.helpers import deep_update


@pytest.fixture
def default_capabilities(request):
    """Firefox workaround to disable baseline FPP for set_window_rect tests.

    This is a hack for Firefox on Linux and Mac. The maximize detection heuristic
    compares window.outerWidth/Height to screen.availWidth/Height. With Firefox's
    baseline fingerprinting protection (bFPP) enabled in Nightly,
    screen.availWidth/Height are spoofed while window dimensions are not,
    causing the heuristic to fail.
    See https://bugzilla.mozilla.org/show_bug.cgi?id=1990514 for details about Linux.
    See https://bugzilla.mozilla.org/show_bug.cgi?id=2016273 for the future fix for Mac.
    """
    capabilities = request.getfixturevalue("default_capabilities")

    if sys.platform.startswith("linux") or sys.platform == "darwin":
        deep_update(
            capabilities,
            {
                "moz:firefoxOptions": {
                    "prefs": {
                        "privacy.baselineFingerprintingProtection": False
                    }
                }
            }
        )

    return capabilities
