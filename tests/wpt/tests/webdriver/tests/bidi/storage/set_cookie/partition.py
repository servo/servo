import pytest
from webdriver.bidi.modules.storage import BrowsingContextPartitionDescriptor, StorageKeyPartitionDescriptor
from .. import assert_cookie_is_set, create_cookie

pytestmark = pytest.mark.asyncio


async def test_partition_context(bidi_session, top_context, test_page, origin, domain_value):
    await bidi_session.browsing_context.navigate(context=top_context["context"], url=test_page, wait="complete")

    source_origin = origin()
    partition = BrowsingContextPartitionDescriptor(top_context["context"])

    set_cookie_result = await bidi_session.storage.set_cookie(
        cookie=create_cookie(domain=domain_value()),
        partition=partition)

    assert set_cookie_result == {
        'partitionKey': {
            'sourceOrigin': source_origin
        },
    }

    await assert_cookie_is_set(bidi_session, domain=domain_value(), partition=partition)


async def test_partition_context_frame(bidi_session, top_context, test_page, origin, domain_value,
                                       inline, test_page_cross_origin_frame):
    frame_url = inline("<div>bar</div>", domain="alt")
    frame_source_origin = origin(domain="alt")
    root_page_url = inline(f"<iframe src='{frame_url}'></iframe>")

    # Navigate to a page with a frame.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=root_page_url,
        wait="complete",
    )

    all_contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    frame_context_id = all_contexts[0]["children"][0]["context"]

    partition = BrowsingContextPartitionDescriptor(frame_context_id)

    set_cookie_result = await bidi_session.storage.set_cookie(
        cookie=create_cookie(domain=domain_value()),
        partition=partition)

    assert set_cookie_result == {
        'partitionKey': {
            'sourceOrigin': frame_source_origin
        },
    }

    await assert_cookie_is_set(bidi_session, domain=domain_value(), partition=partition)


async def test_partition_storage_key_source_origin(bidi_session, test_page, origin, domain_value):
    source_origin = origin()
    partition = StorageKeyPartitionDescriptor(source_origin=source_origin)

    set_cookie_result = await bidi_session.storage.set_cookie(
        cookie=create_cookie(domain=domain_value()),
        partition=partition)

    assert set_cookie_result == {
        'partitionKey': {
            'sourceOrigin': source_origin
        },
    }

    await assert_cookie_is_set(bidi_session, domain=domain_value(), partition=partition)

# TODO: test `test_partition_storage_key_user_context`.
