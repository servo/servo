import pytest

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("domain", ["", "alt"],
                         ids=["same_origin", "cross_origin"])
@pytest.mark.asyncio
async def test_data_persists_after_navigation(bidi_session, top_context,
        setup_network_test, add_data_collector, url, test_page, domain):
    network_events = await setup_network_test(
        events=["network.responseCompleted"])
    events = network_events["network.responseCompleted"]

    # Setup data collector.
    collector = await add_data_collector(
        collector_type="blob",
        data_types=["response"],
        max_encoded_data_size=10_000,
    )

    # Init first navigation.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_page,
        wait="complete",
    )

    # Get request ID.
    request = events[-1]['request']['request']

    # Assert data is available.
    data = await bidi_session.network.get_data(request=request,
                                               collector=collector,
                                               data_type="response")
    assert data["type"] == "string"
    assert "<div>foo</div>" in data["value"]

    # Navigate away from the page.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url('/', domain=domain),
        wait="complete",
    )

    # Assert data is still available.
    data = await bidi_session.network.get_data(request=request,
                                               collector=collector,
                                               data_type="response")
    assert data["type"] == "string"
    assert "<div>foo</div>" in data["value"]


@pytest.mark.asyncio
async def test_data_persists_after_closing(bidi_session, setup_network_test,
        add_data_collector, test_page):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")

    network_events = await setup_network_test(
        events=["network.responseCompleted"])
    events = network_events["network.responseCompleted"]

    # Setup data collector.
    collector = await add_data_collector(
        collector_type="blob",
        data_types=["response"],
        max_encoded_data_size=10_000,
    )

    # Init first navigation.
    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=test_page,
        wait="complete",
    )

    # Get request ID.
    request = events[-1]['request']['request']

    # Assert data is available.
    data = await bidi_session.network.get_data(request=request,
                                               collector=collector,
                                               data_type="response")
    assert data["type"] == "string"
    assert "<div>foo</div>" in data["value"]

    # Close the page.
    await bidi_session.browsing_context.close(context=new_context["context"])

    # Assert data is still available.
    data = await bidi_session.network.get_data(request=request,
                                               collector=collector,
                                               data_type="response")
    assert data["type"] == "string"
    assert "<div>foo</div>" in data["value"]
