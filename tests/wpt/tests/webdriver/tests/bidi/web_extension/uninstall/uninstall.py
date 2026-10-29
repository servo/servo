import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


async def test_uninstall(bidi_session, extension_data):
    # Use "path" as it is supported more widely.
    web_extension = await bidi_session.web_extension.install(
        extension_data={
            "type": "path",
            "path": extension_data["path"],
        }
    )
    await bidi_session.web_extension.uninstall(
        extension=web_extension
    )
    # proof that the uninstall was successful
    with pytest.raises(error.NoSuchWebExtensionException):
        await bidi_session.web_extension.uninstall(
            extension=web_extension
        )
