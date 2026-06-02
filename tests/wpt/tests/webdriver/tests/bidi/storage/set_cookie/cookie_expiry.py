import time
from datetime import datetime, timedelta

import pytest
from webdriver.bidi.modules.storage import CookieFilter
from .. import assert_cookie_is_not_set, assert_partition_key, assert_cookie_is_set, create_cookie

pytestmark = pytest.mark.asyncio


async def test_cookie_expiry_unset(bidi_session, set_cookie, domain_value):
    set_cookie_result = await set_cookie(
        cookie=create_cookie(
            domain=domain_value(),
            expiry=None))

    await assert_partition_key(bidi_session, actual=set_cookie_result["partitionKey"])

    await assert_cookie_is_set(bidi_session, expiry=None, domain=domain_value())


async def test_cookie_expiry_future(bidi_session, set_cookie, domain_value):
    tomorrow = datetime.now() + timedelta(1)
    tomorrow_timestamp = time.mktime(tomorrow.timetuple())

    set_cookie_result = await set_cookie(
        cookie=create_cookie(
            domain=domain_value(),
            expiry=tomorrow_timestamp))

    await assert_partition_key(bidi_session, actual=set_cookie_result["partitionKey"])

    await assert_cookie_is_set(bidi_session, expiry=tomorrow_timestamp, domain=domain_value())


async def test_cookie_expiry_past(bidi_session, set_cookie, domain_value):
    yesterday = datetime.now() - timedelta(1)
    yesterday_timestamp = time.mktime(yesterday.timetuple())

    set_cookie_result = await set_cookie(
        cookie=create_cookie(
            domain=domain_value(),
            expiry=yesterday_timestamp))

    await assert_partition_key(bidi_session, actual=set_cookie_result["partitionKey"])

    await assert_cookie_is_not_set(bidi_session)


async def test_cookie_expiry_future_far(bidi_session, set_cookie, domain_value):
    five_years = datetime.now() + timedelta(days=5 * 365)
    five_years_timestamp = time.mktime(five_years.timetuple())

    # There is a recommended upper limit of 400 days on the allowed expiry
    # value, which user agent can adjust.
    set_cookie_result = await set_cookie(
        cookie=create_cookie(
            domain=domain_value(),
            expiry=five_years_timestamp))

    await assert_partition_key(bidi_session, actual=set_cookie_result["partitionKey"])

    cookies = await bidi_session.storage.get_cookies(filter=CookieFilter(domain=domain_value()))
    assert len(cookies["cookies"]) == 1
    assert cookies["cookies"][0]["expiry"] <= five_years_timestamp

