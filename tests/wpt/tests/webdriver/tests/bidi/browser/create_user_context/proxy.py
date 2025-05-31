import pytest

from .. import get_user_context_ids

pytestmark = pytest.mark.asyncio


@pytest.fixture
def create_user_context_with_proxy(bidi_session, create_user_context):
    async def create_user_context_with_proxy(proxy):
        user_context = await create_user_context(proxy=proxy)

        assert user_context in await get_user_context_ids(bidi_session)

        return user_context

    return create_user_context_with_proxy


async def test_system(create_user_context_with_proxy):
    await create_user_context_with_proxy({
        "proxyType": "system"
    })
    # TODO: check the proxy is actually set.


async def test_autodetect(
        create_user_context_with_proxy):
    await create_user_context_with_proxy({
        "proxyType": "autodetect"
    })
    # TODO: check the proxy is actually set.


async def test_direct(create_user_context_with_proxy):
    await create_user_context_with_proxy({
        "proxyType": "direct"
    })
    # TODO: check the proxy is actually set.


@pytest.mark.parametrize("ftpProxy", [None, "127.0.0.1:21"])
@pytest.mark.parametrize("httpProxy", [None, "127.0.0.1:80"])
@pytest.mark.parametrize("sslProxy", [None, "127.0.0.1:443"])
@pytest.mark.parametrize("noProxy", [None, ["127.0.0.1"]])
@pytest.mark.parametrize("socks", [None, {
    "socksProxy": "127.0.0.1:1080",
    "socksVersion": 5}])
async def test_manual(create_user_context_with_proxy,
        ftpProxy, httpProxy, sslProxy,
        noProxy, socks):
    proxy = {
        "proxyType": "manual"
    }

    if ftpProxy is not None:
        proxy["ftpProxy"] = ftpProxy

    if httpProxy is not None:
        proxy["httpProxy"] = httpProxy

    if sslProxy is not None:
        proxy["sslProxy"] = sslProxy

    if noProxy is not None:
        proxy["noProxy"] = noProxy

    if socks is not None:
        proxy.update(socks)

    await create_user_context_with_proxy(proxy)
    # TODO: check the proxy is actually set.

# TODO: test "proxyType": "pac"
