from typing import Optional
from webdriver.bidi.modules.network import NetworkBytesValue, NetworkStringValue
from webdriver.bidi.modules.storage import PartialCookie, PartitionDescriptor
from .. import any_int, recursive_compare

COOKIE_NAME = 'SOME_COOKIE_NAME'
COOKIE_VALUE = NetworkStringValue('SOME_COOKIE_VALUE')


async def assert_cookie_is_not_set(bidi_session, name: str = COOKIE_NAME):
    """
    Asserts the cookie is not set.
    """
    all_cookies = await bidi_session.storage.get_cookies()
    assert 'cookies' in all_cookies
    assert not any(c for c in all_cookies['cookies'] if c['name'] == name)


async def assert_cookie_is_set(
        bidi_session,
        domain: str,
        name: str = COOKIE_NAME,
        value: str = COOKIE_VALUE,
        path: str = "/",
        http_only: bool = False,
        secure: bool = True,
        same_site: str = 'none',
        expiry: Optional[int] = None,
        partition: Optional[PartitionDescriptor] = None,
):
    """
    Asserts the cookie is set.
    """
    all_cookies = await bidi_session.storage.get_cookies(partition=partition)
    assert 'cookies' in all_cookies
    actual_cookie = next(c for c in all_cookies['cookies'] if c['name'] == name)
    expected_cookie = {
        'domain': domain,
        'httpOnly': http_only,
        'name': name,
        'path': path,
        'sameSite': same_site,
        'secure': secure,
        # Varies depending on the cookie name and value.
        'size': any_int,
        'value': value,
    }
    if expiry is not None:
        expected_cookie['expiry'] = expiry

    recursive_compare(expected_cookie, actual_cookie)


def create_cookie(
        domain: str,
        name: str = COOKIE_NAME,
        value: NetworkBytesValue = COOKIE_VALUE,
        secure: Optional[bool] = True,
        path: Optional[str] = None,
        http_only: Optional[bool] = None,
        same_site: Optional[str] = None,
        expiry: Optional[int] = None,
) -> PartialCookie:
    """
    Creates a cookie with the given or default options.
    """
    return PartialCookie(
        domain=domain,
        name=name,
        value=value,
        path=path,
        http_only=http_only,
        secure=secure,
        same_site=same_site,
        expiry=expiry)
