import pytest
from webdriver.bidi.modules.storage import BrowsingContextPartitionDescriptor, StorageKeyPartitionDescriptor
from .. import assert_cookie_is_set, create_cookie
from ... import recursive_compare

pytestmark = pytest.mark.asyncio


def assert_set_cookie_result(set_cookie_result, partition):
    """
    Asserts the result of `set_cookie` command depending on the partition type.
    """
    if isinstance(partition, BrowsingContextPartitionDescriptor):
        # Browsing context does not require a `sourceOrigin` partition key, but it can be present depending on the
        # browser implementation.
        # `recursive_compare` allows the actual result to be any extension of the expected one.
        recursive_compare({'partitionKey': {}, }, set_cookie_result)
        return
    if isinstance(partition, StorageKeyPartitionDescriptor):
        expected_partition_key = {}
        if "sourceOrigin" in partition:
            # `sourceOrigin` should be in the result, as it was used for setting cookie.
            expected_partition_key["sourceOrigin"] = partition["sourceOrigin"]
        # The specific partition keys can contain other browser-specific keys.
        # `recursive_compare` allows the actual result to be any extension of the expected one.
        recursive_compare({'partitionKey': expected_partition_key}, set_cookie_result)
        return
    assert False, f"Unsupported partition type {type(partition)}."


async def test_partition_context(bidi_session, set_cookie, top_context, test_page, domain_value):
    await bidi_session.browsing_context.navigate(context=top_context["context"], url=test_page, wait="complete")

    partition = BrowsingContextPartitionDescriptor(top_context["context"])
    set_cookie_result = await set_cookie(
        cookie=create_cookie(domain=domain_value()),
        partition=partition)
    assert_set_cookie_result(set_cookie_result, partition)

    await assert_cookie_is_set(bidi_session, domain=domain_value())


async def test_partition_context_frame(bidi_session, set_cookie, top_context, test_page, domain_value, inline):
    frame_url = inline("<div>bar</div>", domain="alt")
    root_page_url = inline(f"<iframe src='{frame_url}'></iframe>")
    root_page_domain = domain_value()

    # Navigate to a page with a frame.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=root_page_url,
        wait="complete",
    )

    all_contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    frame_context_id = all_contexts[0]["children"][0]["context"]

    partition = BrowsingContextPartitionDescriptor(frame_context_id)
    set_cookie_result = await set_cookie(
        cookie=create_cookie(domain=root_page_domain),
        partition=partition)
    assert_set_cookie_result(set_cookie_result, partition)

    await assert_cookie_is_set(bidi_session, domain=root_page_domain)


async def test_partition_storage_key_source_origin(bidi_session, set_cookie, test_page, origin, domain_value):
    source_origin = origin()
    partition = StorageKeyPartitionDescriptor(source_origin=source_origin)

    set_cookie_result = await set_cookie(
        cookie=create_cookie(domain=domain_value()),
        partition=partition)
    assert_set_cookie_result(set_cookie_result, partition)

    await assert_cookie_is_set(bidi_session, domain=domain_value(), partition=partition)

# TODO: test `test_partition_storage_key_user_context`.
