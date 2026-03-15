import pytest

from .. import assert_extension_id

pytestmark = pytest.mark.asyncio


async def test_install_from_base64(bidi_session, install_webextension, extension_data):
    web_extension = await install_webextension(
        extension_data={
            "type": "base64",
            "value": extension_data["base64"],
        }
    )
    assert_extension_id(web_extension, extension_data)


async def test_install_from_path(bidi_session, install_webextension, extension_data):
    web_extension = await install_webextension(
        extension_data={
            "type": "path",
            "path": extension_data["path"],
        }
    )
    assert_extension_id(web_extension, extension_data)


async def test_install_from_archive_path(bidi_session, install_webextension, extension_data):
    web_extension = await install_webextension(
        extension_data={
            "type": "archivePath",
            "path": extension_data["archivePath"],
        }
    )
    assert_extension_id(web_extension, extension_data)
