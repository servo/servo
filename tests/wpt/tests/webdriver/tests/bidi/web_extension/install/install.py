import pytest


@pytest.mark.asyncio
async def test_install_from_base64(bidi_session, extension_data):
    web_extension = await bidi_session.web_extension.install(
        extension_data={
            "type": "base64",
            "value": extension_data["base64"],
        }
    )
    try:
        assert web_extension == extension_data["id"]
    finally:
        # Clean up the extension.
        await bidi_session.web_extension.uninstall(extension=web_extension)


@pytest.mark.asyncio
async def test_install_from_path(bidi_session, extension_data):
    web_extension = await bidi_session.web_extension.install(
        extension_data={
            "type": "path",
            "path": extension_data["path"],
        }
    )
    try:
        assert web_extension == extension_data["id"]
    finally:
        # Clean up the extension.
        await bidi_session.web_extension.uninstall(extension=web_extension)


@pytest.mark.asyncio
async def test_install_from_archive_path(bidi_session, extension_data):
    web_extension = await bidi_session.web_extension.install(
        extension_data={
            "type": "archivePath",
            "path": extension_data["archivePath"],
        }
    )
    try:
        assert web_extension == extension_data["id"]
    finally:
        # Clean up the extension.
        await bidi_session.web_extension.uninstall(extension=web_extension)
