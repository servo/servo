from datetime import datetime, timedelta, timezone
from typing import Optional
from webdriver.bidi.modules.network import NetworkBytesValue, NetworkStringValue
from webdriver.bidi.modules.storage import PartialCookie, PartitionDescriptor, BrowsingContextPartitionDescriptor
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


def generate_expiry_date(day_diff=1):
    return (
        (datetime.now(timezone.utc) + timedelta(days=day_diff))
        .replace(microsecond=0)
        .replace(tzinfo=timezone.utc)
    )


def format_expiry_string(date):
    # same formatting as Date.toUTCString() in javascript
    utc_string_format = "%a, %d %b %Y %H:%M:%S GMT"
    return date.strftime(utc_string_format)


async def get_default_partition_key(bidi_session, context=None):
    if context is None:
        result = await bidi_session.storage.get_cookies()
    else:
        result = await bidi_session.storage.get_cookies(partition=BrowsingContextPartitionDescriptor(context))
    return result['partitionKey']


async def assert_partition_key(bidi_session, actual, expected = {}, context=None):
    expected = {
        **(await get_default_partition_key(bidi_session, context)),
        **expected
    }
    recursive_compare(expected, actual)
