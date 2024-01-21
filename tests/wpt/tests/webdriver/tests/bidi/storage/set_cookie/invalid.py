import pytest
from .. import create_cookie
import webdriver.bidi.error as error
from webdriver.bidi.modules.network import NetworkStringValue
from webdriver.bidi.modules.storage import BrowsingContextPartitionDescriptor, StorageKeyPartitionDescriptor

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("domain", [None, False, 42, {}, []])
async def test_cookie_domain_invalid_type(bidi_session, test_page, domain):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain))


@pytest.mark.parametrize("expiry", [False, "SOME_STRING_VALUE", {}, []])
async def test_cookie_expiry_invalid_type(bidi_session, test_page, domain_value, expiry):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value(), expiry=expiry))


@pytest.mark.parametrize("http_only", [42, "SOME_STRING_VALUE", {}, []])
async def test_cookie_http_only_invalid_type(bidi_session, test_page, domain_value, http_only):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value(), http_only=http_only))


@pytest.mark.parametrize(
    "name",
    [
        " cookieName",
        "cookie=name",
        "cookie\tname",
        "cookie\nname",
        "cookie\x01name",
        "cookie\x0Fname",
        "cookie;name",
    ])
async def test_cookie_name_invalid_value(bidi_session, test_page, domain_value, name):
    with pytest.raises(error.UnableToSetCookieException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value(), name=name))


@pytest.mark.parametrize("name", [None, False, 42, {}, []])
async def test_cookie_name_invalid_type(bidi_session, test_page, domain_value, name):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value(), name=name))


@pytest.mark.parametrize(
    "path",
    [
        ""
        "no_leading_forward_slash"
    ]
)
async def test_cookie_path_invalid_value(bidi_session, test_page, domain_value, path):
    with pytest.raises(error.UnableToSetCookieException):
        await bidi_session.storage.set_cookie(
            cookie=create_cookie(domain=domain_value(), path=path))


@pytest.mark.parametrize("path", [False, 42, {}, []])
async def test_cookie_path_invalid_type(bidi_session, test_page, domain_value, path):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(
            cookie=create_cookie(domain=domain_value(), path=path))


@pytest.mark.parametrize("same_site", ["", "INVALID_SAME_SITE_STATE"])
async def test_cookie_same_site_invalid_value(bidi_session, test_page, domain_value, same_site):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value(), same_site=same_site))


@pytest.mark.parametrize("same_site", [42, False, {}, []])
async def test_cookie_same_site_invalid_type(bidi_session, test_page, domain_value, same_site):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value(), same_site=same_site))


@pytest.mark.parametrize("secure", [42, "SOME_STRING_VALUE", {}, []])
async def test_cookie_secure_invalid_type(bidi_session, test_page, domain_value, secure):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value(), secure=secure))


@pytest.mark.parametrize("value", [None, False, 42, "SOME_STRING_VALUE", {}, {"type": "SOME_INVALID_TYPE"}, []])
async def test_cookie_value_invalid_type(bidi_session, test_page, domain_value, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value(), value=value))


@pytest.mark.parametrize(
    "str_value",
    [
        "value\twith\ttab",
        "value\nwith\nnewline",
        "value;with;semicolon",
    ])
async def test_cookie_value_string_invalid_value(bidi_session, test_page, domain_value, str_value):
    value = NetworkStringValue(str_value)

    with pytest.raises(error.UnableToSetCookieException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value(), value=value))


@pytest.mark.parametrize("str_value", [None, False, 42, {}, []])
async def test_cookie_value_string_invalid_type(bidi_session, test_page, domain_value, str_value):
    value = NetworkStringValue(str_value)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value(), value=value))


@pytest.mark.parametrize("partition", [42, False, "SOME_STRING_VALUE", {}, {"type": "SOME_INVALID_TYPE"}, []])
async def test_partition_invalid_type(bidi_session, test_page, domain_value, partition):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value()), partition=partition)


@pytest.mark.parametrize("browsing_context", [None, 42, False, {}, []])
async def test_partition_context_invalid_type(bidi_session, test_page, origin, domain_value, browsing_context):
    partition = BrowsingContextPartitionDescriptor(browsing_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value()), partition=partition)


async def test_partition_context_unknown(bidi_session, test_page, origin, domain_value):
    partition = BrowsingContextPartitionDescriptor("UNKNOWN_CONTEXT")

    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value()), partition=partition)


@pytest.mark.parametrize("source_origin", [42, False, {}, []])
async def test_partition_storage_key_source_origin_invalid_type(bidi_session, test_page, origin, domain_value,
                                                                source_origin):
    partition = StorageKeyPartitionDescriptor(source_origin=source_origin)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value()), partition=partition)


@pytest.mark.parametrize("user_context", [42, False, {}, []])
async def test_partition_storage_key_user_context_invalid_type(bidi_session, test_page, origin, domain_value,
                                                               user_context):
    partition = StorageKeyPartitionDescriptor(user_context=user_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.storage.set_cookie(cookie=create_cookie(domain=domain_value()), partition=partition)

# TODO: test `test_cookie_domain_invalid_value`.
# TODO: test `test_partition_storage_key_user_context_unknown`.
# TODO: test `test_partition_storage_key_user_context_invalid_type`.
# TODO: test `test_cookie_value_missing`.
# TODO: test `test_cookie_value_base64_invalid_type`.
# TODO: test `test_cookie_value_base64_invalid_value`.
