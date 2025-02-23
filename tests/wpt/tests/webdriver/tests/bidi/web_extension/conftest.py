import pytest
from tests.support.helpers import get_addon_path, get_base64_for_addon_file


ADDON_DATA = {
    "firefox": {
        "id": "1FC7D53C-0B0A-49E7-A8C0-47E77496A919@web-platform-tests.org",
        "path": get_addon_path("firefox/unpacked/"),
        "archivePath": get_addon_path("firefox/signed.xpi"),
        "archivePathInvalid": get_addon_path("firefox/invalid.xpi"),
        "base64": get_base64_for_addon_file("firefox/signed.xpi"),
    }
}


@pytest.fixture
def addon_data(current_session):
    browser_name = current_session.capabilities["browserName"]

    return ADDON_DATA[browser_name]
