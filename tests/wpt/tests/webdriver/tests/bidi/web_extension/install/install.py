import pytest


@pytest.mark.asyncio
async def test_install_from_base64(bidi_session, addon_data):
    web_extension = await bidi_session.web_extension.install(
        extension_data={
            "type": "base64",
            "value": addon_data["base64"],
        }
    )
    try:
        assert web_extension == addon_data["id"]
    finally:
        # Clean up the addon.
        await bidi_session.web_extension.uninstall(extension=web_extension)


@pytest.mark.asyncio
async def test_install_from_path(bidi_session, addon_data):
    web_extension = await bidi_session.web_extension.install(
        extension_data={
            "type": "path",
            "path": addon_data["path"],
        }
    )
    try:
        assert web_extension == addon_data["id"]
    finally:
        # Clean up the addon.
        await bidi_session.web_extension.uninstall(extension=web_extension)


@pytest.mark.asyncio
async def test_install_from_archive_path(bidi_session, addon_data):
    web_extension = await bidi_session.web_extension.install(
        extension_data={
            "type": "archivePath",
            "path": addon_data["archivePath"],
        }
    )
    try:
        assert web_extension == addon_data["id"]
    finally:
        # Clean up the addon.
        await bidi_session.web_extension.uninstall(extension=web_extension)
