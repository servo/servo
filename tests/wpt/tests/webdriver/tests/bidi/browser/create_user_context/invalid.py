import pytest

import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


# Even though the user context is not expected to be created, if the user agent
# under the test does not support the parameter, the validation will not fail
# and unexpected user context will be created and will not be closed. Using
# `create_user_context` fixture guarantees the mistakenly created user context
# is destroyed.

@pytest.mark.parametrize("value", [42, "foo", {}, []])
async def test_accept_insecure_certs_invalid_type(create_user_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await create_user_context(accept_insecure_certs=value)


@pytest.mark.parametrize("value", [False, 42, "foo", []])
async def test_proxy_invalid_type(create_user_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await create_user_context(
            proxy=value)


@pytest.mark.parametrize("value", [{}])
async def test_proxy_invalid_value(create_user_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await create_user_context(
            proxy=value)


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_proxy_proxy_type_invalid_type(create_user_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await create_user_context(
            proxy={
                "proxyType": value
            })


async def test_proxy_proxy_type_invalid_value(create_user_context):
    with pytest.raises(error.InvalidArgumentException):
        await create_user_context(
            proxy={
                "proxyType": "SOME_UNKNOWN_TYPE"
            })


async def test_proxy_proxy_type_manual_socks_version_without_socks_proxy(
        create_user_context):
    with pytest.raises(error.InvalidArgumentException):
        await create_user_context(
            proxy={
                "proxyType": "manual",
                "socksVersion": 0
            })


async def test_proxy_proxy_type_manual_socks_proxy_without_socks_version(
        create_user_context):
    with pytest.raises(error.InvalidArgumentException):
        await create_user_context(
            proxy={
                "proxyType": "manual",
                "socksProxy": "127.0.0.1:1080"
            })
