import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, False, 42, [], ""])
async def test_params_extension_data_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.web_extension.install(
            extension_data=value
        )


async def test_params_extension_data_invalid_value(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.web_extension.install(
            extension_data={}
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_extension_data_type_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.web_extension.install(
            extension_data={ "type": value }
        )


@pytest.mark.parametrize("value", ["", "unknown-type"])
async def test_params_extension_data_type_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.web_extension.install(
            extension_data={ "type": value }
        )


@pytest.mark.parametrize("value", ["path", "archivePath"])
async def test_params_extension_data_path_missing(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.web_extension.install(
            extension_data={ "type": value }
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
@pytest.mark.parametrize("data_type", ["path", "archivePath"])
async def test_params_extension_data_path_invalid_type(bidi_session, data_type, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.web_extension.install(
            extension_data={ "type": data_type, "path": value }
        )


@pytest.mark.parametrize("value", ["", "invalid path"])
@pytest.mark.parametrize("data_type", ["path", "archivePath"])
async def test_params_extension_data_path_invalid_value(bidi_session, data_type, value):
    with pytest.raises(error.UnknownErrorException):
        await bidi_session.web_extension.install(
            extension_data={ "type": data_type, "path": value }
        )


async def test_params_extension_data_archive_path_invalid_webextension(bidi_session, extension_data):
    with pytest.raises(error.InvalidWebExtensionException):
        await bidi_session.web_extension.install(
            extension_data={"type": "archivePath",
                           "path": extension_data["archivePathInvalid"]}
        )


async def test_params_extension_data_path_invalid_webextension(bidi_session, extension_data):
    with pytest.raises(error.InvalidWebExtensionException):
        await bidi_session.web_extension.install(
            extension_data={"type": "path", "path": extension_data["archivePath"]}
        )


async def test_params_extension_data_value_missing(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.web_extension.install(
            extension_data={ "type": "base64" }
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_extension_data_value_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.web_extension.install(
            extension_data={ "type": "base64", "value": value }
        )


async def test_params_extension_data_value_invalid_value(bidi_session):
    with pytest.raises(error.UnknownErrorException):
        await bidi_session.web_extension.install(
            extension_data={ "type": "base64", "value": "not a base64" }
        )


@pytest.mark.parametrize("value", ["", "dGVzdA=="])
async def test_params_extension_data_value_invalid_webextension(bidi_session, value):
    with pytest.raises(error.InvalidWebExtensionException):
        await bidi_session.web_extension.install(
            extension_data={ "type": "base64", "value": value }
        )
