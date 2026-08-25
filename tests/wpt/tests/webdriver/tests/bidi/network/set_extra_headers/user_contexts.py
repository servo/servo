import pytest

from tests.bidi.network.set_extra_headers import SOME_HEADER_NAME, \
    USER_CONTEXT_HEADER, USER_CONTEXT_VALUE, GLOBAL_HEADER, GLOBAL_VALUE
from tests.bidi.network.set_extra_headers.conftest import prepare_context

pytestmark = pytest.mark.asyncio


async def test_isolation(bidi_session, create_user_context,
        assert_header_present, assert_header_not_present, affected_user_context,
        not_affected_user_context, prepare_context, set_extra_headers):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)
    await prepare_context(affected_context)
    not_affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=not_affected_user_context)
    await prepare_context(not_affected_context)

    await assert_header_not_present(affected_context, SOME_HEADER_NAME)
    await assert_header_not_present(not_affected_context, SOME_HEADER_NAME)

    await set_extra_headers(headers=[USER_CONTEXT_HEADER],
                            user_contexts=[affected_user_context])

    await assert_header_present(affected_context, SOME_HEADER_NAME,
                                USER_CONTEXT_VALUE)
    await assert_header_not_present(not_affected_context, SOME_HEADER_NAME)

    another_affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)
    await prepare_context(another_affected_context)
    another_not_affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=not_affected_user_context)
    await prepare_context(another_not_affected_context)
    await assert_header_present(another_affected_context, SOME_HEADER_NAME,
                                USER_CONTEXT_VALUE)
    await assert_header_not_present(another_not_affected_context,
                                    SOME_HEADER_NAME)

    await set_extra_headers(headers=[], user_contexts=[affected_user_context])

    await assert_header_not_present(affected_context, SOME_HEADER_NAME)
    await assert_header_not_present(not_affected_context, SOME_HEADER_NAME)
    await assert_header_not_present(another_affected_context, SOME_HEADER_NAME)
    await assert_header_not_present(another_not_affected_context,
                                    SOME_HEADER_NAME)


@pytest.mark.parametrize("domain", ["", "alt"],
                         ids=["same_origin", "cross_origin"])
async def test_frame(bidi_session, url, assert_header_present,
        assert_header_not_present, set_extra_headers,
        create_iframe, domain, prepare_context, affected_user_context):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)
    # Change top-level frame's origin to allow for fetch from the iframe.
    await prepare_context(affected_context, domain)
    iframe_id = await create_iframe(affected_context, url('/', domain=""));

    await assert_header_not_present(iframe_id, SOME_HEADER_NAME)

    await set_extra_headers(headers=[USER_CONTEXT_HEADER],
                            user_contexts=[affected_user_context])

    await assert_header_present(iframe_id, SOME_HEADER_NAME,
                                USER_CONTEXT_VALUE)

    await set_extra_headers(headers=[], user_contexts=[affected_user_context])

    await assert_header_not_present(iframe_id, SOME_HEADER_NAME)


async def test_overrides_global(bidi_session, url, assert_header_present,
        assert_header_not_present, set_extra_headers, top_context,
        prepare_context, affected_user_context):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)
    await prepare_context(affected_context)

    await assert_header_not_present(affected_context, SOME_HEADER_NAME)

    await set_extra_headers(headers=[GLOBAL_HEADER])

    await assert_header_present(affected_context, SOME_HEADER_NAME,
                                GLOBAL_VALUE)

    await set_extra_headers(headers=[USER_CONTEXT_HEADER],
                            user_contexts=[affected_user_context])

    await assert_header_present(affected_context, SOME_HEADER_NAME,
                                USER_CONTEXT_VALUE)

    await set_extra_headers(headers=[],
                            user_contexts=[affected_user_context])

    await assert_header_present(affected_context, SOME_HEADER_NAME,
                                GLOBAL_VALUE)

    await set_extra_headers(headers=[])

    await assert_header_not_present(affected_context, SOME_HEADER_NAME)
