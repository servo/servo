import sys

import pytest
from tests.support.helpers import deep_update


@pytest.fixture
def default_capabilities(request):
    """Firefox/Linux workaround to disable baseline FPP for fullscreen tests.

    This is a hack for Firefox on Linux only. The maximize detection heuristic
    compares window.outerWidth/Height to screen.availWidth/Height. With Firefox's
    baseline fingerprinting protection (bFPP) enabled in Nightly,
    screen.availWidth/Height are spoofed while window dimensions are not,
    causing the heuristic to fail.
    See https://bugzilla.mozilla.org/show_bug.cgi?id=1990514 for details.
    """
    capabilities = request.getfixturevalue("default_capabilities")

    if sys.platform.startswith("linux"):
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
