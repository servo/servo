import pytest
import webdriver.bidi.error as error
from webdriver.bidi.modules.network import NetworkBase64Value, NetworkStringValue
from webdriver.bidi.modules.storage import (
    BrowsingContextPartitionDescriptor,
    CookieFilter,
    StorageKeyPartitionDescriptor,
)

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [False, 42, "foo", []])
async def test_params_filter_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(filter=value)


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_filter_domain_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(filter=CookieFilter(domain=value))


@pytest.mark.parametrize("value", [False, "foo", {}, [], -1, 0.5])
async def test_params_filter_expiry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(filter=CookieFilter(expiry=value))


@pytest.mark.parametrize("value", ["foo", {}, [], 42])
async def test_params_filter_http_only_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(filter=CookieFilter(http_only=value))


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_filter_name_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(filter=CookieFilter(name=value))


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_filter_path_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(filter=CookieFilter(path=value))


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_filter_same_site_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(filter=CookieFilter(same_site=value))


@pytest.mark.parametrize("value", ["", "INVALID_SAME_SITE_STATE"])
async def test_params_filter_same_site_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(filter=CookieFilter(same_site=value))


@pytest.mark.parametrize("value", ["foo", {}, [], 42])
async def test_params_filter_secure_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(filter=CookieFilter(secure=value))


@pytest.mark.parametrize("value", [False, "foo", {}, [], -1, 0.5])
async def test_params_filter_size_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(filter=CookieFilter(size=value))


@pytest.mark.parametrize("value", [False, 42, "foo", []])
async def test_params_filter_value_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(filter=CookieFilter(value=value))


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_filter_value_type_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(
            filter=CookieFilter(value={"type": value})
        )


async def test_params_filter_value_type_invalid_value(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(
            filter=CookieFilter(value={"type": "foo"})
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_filter_value_base64_type_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(
            filter=CookieFilter(value=NetworkBase64Value(value))
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_filter_value_string_type_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(
            filter=CookieFilter(value=NetworkStringValue(value))
        )


@pytest.mark.parametrize("value", [False, 42, "foo", []])
async def test_params_partition_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(partition=value)


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_partition_type_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(partition={"type": value})


async def test_params_partition_type_invalid_value(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(partition={"type": "foo"})


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_partition_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(
            partition=BrowsingContextPartitionDescriptor(context=value)
        )


async def test_partition_invalid_context(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.storage.get_cookies(
            partition=BrowsingContextPartitionDescriptor("foo")
        )


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_partition_source_origin_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(
            partition=StorageKeyPartitionDescriptor(source_origin=value)
        )


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_partition_user_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.get_cookies(
            partition=StorageKeyPartitionDescriptor(user_context=value)
        )


async def test_params_partition_user_context_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.storage.get_cookies(
            partition=StorageKeyPartitionDescriptor(user_context="foo")
        )
