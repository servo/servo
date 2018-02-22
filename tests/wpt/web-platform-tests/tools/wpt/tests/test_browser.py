import os

from tools.wpt import browser

here = os.path.dirname(__file__)


def test_firefox_nightly_link():
    expected = ("https://archive.mozilla.org/pub/firefox/nightly/latest-mozilla-central/"
                "firefox-60.0a1.en-US.linux-x86_64.tar.bz2")
    with open(os.path.join(here, "latest_mozilla_central.txt")) as index:
        fx = browser.Firefox()
        assert fx.get_nightly_link(index.read(), "linux-x86_64") == expected
