import pytest

pytestmark = pytest.mark.asyncio


async def test_set_and_remove(top_context,
        prepare_context, get_headers_methods_invariant, set_extra_headers):
    await prepare_context(top_context)

    original_headers = await get_headers_methods_invariant(top_context)
    await set_extra_headers(
        headers=[{
            "name": "some_header_name",
            "value": {
                "type": "string",
                "value": "some_header_value"
            }}],
        contexts=[top_context["context"]])
    new_headers = await get_headers_methods_invariant(top_context)
    assert new_headers["some_header_name"] == ["some_header_value"]

    await set_extra_headers(headers=[], contexts=[top_context["context"]])
    assert original_headers == await get_headers_methods_invariant(top_context)


async def test_set_and_unsubscribe_from_network(bidi_session, top_context,
        prepare_context, get_headers_methods_invariant, set_extra_headers, subscribe_events):
    await prepare_context(top_context)

    await subscribe_events(events=["network.beforeRequestSent"])

    await set_extra_headers(
        headers=[{
            "name": "some_header_name",
            "value": {
                "type": "string",
                "value": "some_header_value"
            }}],
        contexts=[top_context["context"]])

    # Make sure extra headers are still added after unsubscribing from network
    # events.
    await bidi_session.session.unsubscribe(events=["network.beforeRequestSent"])

    new_headers = await get_headers_methods_invariant(top_context)
    assert new_headers["some_header_name"] == ["some_header_value"]


async def test_multiple_headers(top_context,
        prepare_context, get_headers_methods_invariant, set_extra_headers):
    await prepare_context(top_context)

    original_headers = await get_headers_methods_invariant(top_context)
    await set_extra_headers(
        headers=[{
            "name": "some_header_name",
            "value": {
                "type": "string",
                "value": "some_header_value_1"
            }
        }, {
            "name": "some_header_name",
            "value": {
                "type": "string",
                "value": "some_header_value_2"
            }
        }, {
            "name": "another_header_name",
            "value": {
                "type": "string",
                "value": "another_header_value"
            }
        }],
        contexts=[top_context["context"]])
    new_headers = await get_headers_methods_invariant(top_context)
    assert new_headers["some_header_name"] == ["some_header_value_2"]
    assert new_headers["another_header_name"] == ["another_header_value"]

    await set_extra_headers(headers=[], contexts=[top_context["context"]])
    assert original_headers == await get_headers_methods_invariant(top_context)


async def test_headers_merged(bidi_session, prepare_context, set_extra_headers,
        assert_header_present, affected_user_context):
    affected_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=affected_user_context)
    await prepare_context(affected_context)

    await set_extra_headers(
        headers=[{
            "name": "some_context_name",
            "value": {
                "type": "string",
                "value": "some_context_value"
            }}],
        contexts=[affected_context["context"]])

    await set_extra_headers(
        headers=[{
            "name": "some_user_context_name",
            "value": {
                "type": "string",
                "value": "some_user_context_value"
            }}],
        user_contexts=[affected_user_context])

    await set_extra_headers(
        headers=[{
            "name": "some_global_name",
            "value": {
                "type": "string",
                "value": "some_global_value"
            }}])

    await assert_header_present(affected_context, "some_context_name",
                                "some_context_value")
    await assert_header_present(affected_context, "some_user_context_name",
                                "some_user_context_value")
    await assert_header_present(affected_context, "some_global_name",
                                "some_global_value")
