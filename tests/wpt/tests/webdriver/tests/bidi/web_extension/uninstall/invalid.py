import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


async def test_uninstall_missing_extension(bidi_session):
    with pytest.raises(error.NoSuchWebExtensionException):
        await bidi_session.web_extension.uninstall(extension="test")


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_extension_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.web_extension.uninstall(extension=value)


@pytest.mark.parametrize("value", ["", "unknown-ext"], ids=["empty", "unknown"])
async def test_params_extension_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchWebExtensionException):
        await bidi_session.web_extension.uninstall(extension=value)
