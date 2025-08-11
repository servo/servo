import pytest
import webdriver.bidi.error as error

from .. import PAGE_EMPTY_TEXT, RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


async def test_max_encoded_data_size(
    bidi_session,
    add_data_collector,
    setup_network_test,
    wait_for_event,
    wait_for_future_safe,
    url,
    fetch,
):
    context = await bidi_session.browsing_context.create(type_hint="tab")
    await setup_network_test(
        events=[RESPONSE_COMPLETED_EVENT], context=context["context"]
    )

    small_collector = await add_data_collector(max_encoded_data_size=1)
    big_collector = await add_data_collector(max_encoded_data_size=100000)

    # Trigger a request in `context`
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(url(PAGE_EMPTY_TEXT), context=context)
    event = await wait_for_future_safe(on_response_completed)
    request = event["request"]["request"]

    # request data can be retrieved with big_collector or with no collector
    # argument
    await bidi_session.network.get_data(
        request=request, data_type="response", collector=big_collector
    )
    await bidi_session.network.get_data(
        request=request, data_type="response"
    )
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(
            request=request, data_type="response", collector=small_collector
        )

    # Remove big_collector and check the collected data can no longer be accessed
    await bidi_session.network.remove_data_collector(collector=big_collector)
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(
            request=request, data_type="response"
        )
