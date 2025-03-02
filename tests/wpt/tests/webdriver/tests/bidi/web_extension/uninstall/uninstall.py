import pytest
import webdriver.bidi.error as error


@pytest.mark.asyncio
async def test_uninstall(bidi_session, extension_data):
    web_extension = await bidi_session.web_extension.install(
        extension_data={
            "type": "base64",
            "value": extension_data["base64"],
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
