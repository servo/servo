import pytest
import webdriver.bidi.error as error

from .. import PAGE_EMPTY_TEXT, RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


async def test_multiple_contexts(
    bidi_session,
    add_data_collector,
    setup_network_test,
    wait_for_event,
    wait_for_future_safe,
    url,
    fetch,
):
    context = await bidi_session.browsing_context.create(type_hint="tab")
    other_context = await bidi_session.browsing_context.create(type_hint="tab")
    await setup_network_test(
        events=[RESPONSE_COMPLETED_EVENT], context=context["context"]
    )
    await setup_network_test(
        events=[RESPONSE_COMPLETED_EVENT], context=other_context["context"]
    )

    collector = await add_data_collector(contexts=[context["context"]])
    other_collector = await add_data_collector(contexts=[other_context["context"]])
    both_contexts_collector = await add_data_collector(
        contexts=[context["context"], other_context["context"]]
    )

    # Trigger a request in `context`
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(url(PAGE_EMPTY_TEXT), context=context)
    event = await wait_for_future_safe(on_response_completed)
    request = event["request"]["request"]

    # Trigger a request in `other_context`
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(url(PAGE_EMPTY_TEXT), context=other_context)
    event = await wait_for_future_safe(on_response_completed)
    other_request = event["request"]["request"]

    # request data can be retrieved from collector or both_contexts_collectors
    await bidi_session.network.get_data(
        request=request, data_type="response", collector=collector
    )
    await bidi_session.network.get_data(
        request=request, data_type="response", collector=both_contexts_collector
    )
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(
            request=request, data_type="response", collector=other_collector
        )

    # other_request data can be retrieved from other_collector or both_contexts_collectors
    await bidi_session.network.get_data(
        request=other_request, data_type="response", collector=other_collector
    )
    await bidi_session.network.get_data(
        request=other_request, data_type="response", collector=both_contexts_collector
    )
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(
            request=other_request, data_type="response", collector=collector
        )


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_iframe(
    bidi_session,
    add_data_collector,
    setup_network_test,
    wait_for_event,
    wait_for_future_safe,
    url,
    fetch,
    domain,
    inline,
    new_tab,
):
    iframe_url = inline("<div id='in-iframe'>foo</div>", domain=domain)
    page_url = inline(f"<iframe src='{iframe_url}'></iframe>")

    await setup_network_test(
        events=[RESPONSE_COMPLETED_EVENT], context=new_tab["context"], test_url=page_url
    )

    collector = await add_data_collector(contexts=[new_tab["context"]])

    # Trigger a fetch request in the iframe context.
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(url(PAGE_EMPTY_TEXT), context=new_tab)
    event = await wait_for_future_safe(on_response_completed)
    request = event["request"]["request"]

    # Check that the data for the iframe request has been collected.
    await bidi_session.network.get_data(request=request, data_type="response")
