import pytest
from webdriver.error import TimeoutException
pytestmark = pytest.mark.asyncio
@pytest.mark.asyncio
async def test_speculation_rules_generate_ready_events(
    bidi_session, subscribe_events, new_tab, url, wait_for_events, add_speculation_rules_and_link
):
    '''Test that speculation rules generate prefetch events with proper pending->ready sequence.'''
    await subscribe_events(events=["speculation.prefetchStatusUpdated"])
    test_url = url("/common/blank.html", protocol="https")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="none",
    )
    # Add speculation rules to trigger immediate prefetching
    prefetch_target = url("/common/dummy.xml", protocol="https")
    speculation_rules = f'''{{
        "prefetch": [{{
            "where": {{ "href_matches": "{prefetch_target}" }},
            "eagerness": "immediate"
        }}]
    }}'''
    # Set up event waiter before triggering prefetch
    with wait_for_events(["speculation.prefetchStatusUpdated"]) as waiter:
        # Add speculation rules and link to trigger prefetching
        await add_speculation_rules_and_link(new_tab, speculation_rules, prefetch_target)
        # Wait for pending and ready events
        events = await waiter.get_events(lambda events: len(events) == 2)
    # Verify all events have correct structure and sequence
    assert events == [
        ("speculation.prefetchStatusUpdated", {
            "url": prefetch_target,
            "status": "pending",
            "context": new_tab["context"]
        }),
        ("speculation.prefetchStatusUpdated", {
            "url": prefetch_target,
            "status": "ready",
            "context": new_tab["context"]
        })
    ], f"Events don't match expected sequence: {events}"
@pytest.mark.asyncio
async def test_speculation_rules_generate_events_with_navigation(
    bidi_session, subscribe_events, new_tab, url, wait_for_events, add_speculation_rules_and_link
):
    '''Test that speculation rules generate prefetch events with navigation and success event after using prefetched page.'''
    await subscribe_events(events=["speculation.prefetchStatusUpdated"])
    test_url = url("/common/blank.html", protocol="https")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="none",
    )
    # Add speculation rules to trigger immediate prefetching
    prefetch_target = url("/common/dummy.xml", protocol="https")
    speculation_rules = f'''{{
        "prefetch": [{{
            "where": {{ "href_matches": "{prefetch_target}" }},
            "eagerness": "immediate"
        }}]
    }}'''
    # Set up event waiter before triggering prefetch - capture all events through navigation
    with wait_for_events(["speculation.prefetchStatusUpdated"]) as waiter:
        # Add speculation rules and link to trigger prefetching
        await add_speculation_rules_and_link(new_tab, speculation_rules, prefetch_target)
        # Wait for pending and ready events first
        events = await waiter.get_events(lambda events: len(events) >= 2)
    # Verify we got pending and ready events
    assert events == [
        ("speculation.prefetchStatusUpdated", {
            "url": prefetch_target,
            "status": "pending",
            "context": new_tab["context"]
        }),
        ("speculation.prefetchStatusUpdated", {
            "url": prefetch_target,
            "status": "ready",
            "context": new_tab["context"]
        })
    ], f"Events don't match expected sequence: {events}"
    with wait_for_events(["speculation.prefetchStatusUpdated"]) as waiter:
        # Now navigate to the prefetched page to potentially trigger success event
        # Navigate by clicking the link (user-initiated navigation to trigger success event)
        await bidi_session.script.evaluate(
            expression='''
                const prefetchLink = document.getElementById('prefetch-page');
                if (prefetchLink) {
                    prefetchLink.click();
                }
            ''',
            target={"context": new_tab["context"]},
            await_promise=False
        )
        # Wait for success event after navigation to the prefetched page
        success_event = await waiter.get_events(lambda events: len(events) >= 1)
    # Verify success event has correct structure and sequence
    assert success_event == [
        ("speculation.prefetchStatusUpdated", {
            "url": prefetch_target,
            "status": "success",
            "context": new_tab["context"]
        })
    ], f"Success event doesn't match expected sequence: {success_event}"
@pytest.mark.asyncio
async def test_speculation_rules_generate_failure_events(
    bidi_session, subscribe_events, new_tab, url, wait_for_events, add_speculation_rules_and_link
):
    '''Test that speculation rules generate pending and failure events for failed prefetch.'''
    await subscribe_events(events=["speculation.prefetchStatusUpdated"])
    test_url = url("/common/blank.html", protocol="https")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="none",
    )
    # Create a target that will return 404 - use a non-existent path
    failed_target = url("/nonexistent/path/that/will/404.xml", protocol="https")
    # Add speculation rules to trigger immediate prefetching of 404 page
    speculation_rules = f'''{{
        "prefetch": [{{
            "where": {{ "href_matches": "{failed_target}" }},
            "eagerness": "immediate"
        }}]
    }}'''
    # Set up event waiter before triggering prefetch
    with wait_for_events(["speculation.prefetchStatusUpdated"]) as waiter:
        # Add speculation rules and link to trigger prefetching
        await add_speculation_rules_and_link(new_tab, speculation_rules, failed_target)
        # Wait for events (pending and failure)
        events = await waiter.get_events(lambda events: len(events) >= 2)
    # Verify all events have correct structure and sequence
    assert events == [
        ("speculation.prefetchStatusUpdated", {
            "url": failed_target,
            "status": "pending",
            "context": new_tab["context"]
        }),
        ("speculation.prefetchStatusUpdated", {
            "url": failed_target,
            "status": "failure",
            "context": new_tab["context"]
        })
    ], f"Events don't match expected sequence: {events}"
@pytest.mark.asyncio
async def test_subscribe_unsubscribe_event_emission(
    bidi_session, subscribe_events, new_tab, url, wait_for_events, add_speculation_rules_and_link
):
    '''Test that events are emitted when subscribed and not emitted when unsubscribed.'''
    test_url = url("/common/blank.html", protocol="https")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    prefetch_target = url("/common/dummy.xml", protocol="https")
    speculation_rules = f'''{{
        "prefetch": [{{
            "where": {{ "href_matches": "{prefetch_target}" }},
            "eagerness": "immediate"
        }}]
    }}'''
    # Phase 1: Subscribe to specific event and assert events are emitted
    subscription_result = await subscribe_events(events=["speculation.prefetchStatusUpdated"])
    subscription_id = subscription_result["subscription"]
    # Trigger prefetch and collect events
    with wait_for_events(["speculation.prefetchStatusUpdated"]) as waiter:
        await add_speculation_rules_and_link(new_tab, speculation_rules, prefetch_target)
        # Wait for events to be emitted
        events = await waiter.get_events(lambda events: len(events) >= 2)
    # Phase 2: Unsubscribe and assert events are NOT emitted
    await bidi_session.session.unsubscribe(subscriptions=[subscription_id])
    # Reload the page to get a clean state
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    # Trigger another prefetch after unsubscribing
    prefetch_target_2 = url("/common/square.png", protocol="https")
    speculation_rules_2 = f'''{{
        "prefetch": [{{
            "where": {{ "href_matches": "{prefetch_target_2}" }},
            "eagerness": "immediate"
        }}]
    }}'''
    # Set up waiter but don't expect any events this time
    with wait_for_events(["speculation.prefetchStatusUpdated"]) as waiter:
        await add_speculation_rules_and_link(new_tab, speculation_rules_2, prefetch_target_2)
        with pytest.raises(TimeoutException):
            await waiter.get_events(lambda events: len(events) >= 1, timeout=0.5)
@pytest.mark.asyncio
async def test_subscribe_unsubscribe_module_subscription(
    bidi_session, subscribe_events, new_tab, url, wait_for_events, add_speculation_rules_and_link
):
    '''Test that module subscription ('speculation') works for subscribe/unsubscribe.'''
    test_url = url("/common/blank.html", protocol="https")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    prefetch_target = url("/common/dummy.xml", protocol="https")
    speculation_rules = f'''{{
        "prefetch": [{{
            "where": {{ "href_matches": "{prefetch_target}" }},
            "eagerness": "immediate"
        }}]
    }}'''
    # Phase 1: Subscribe to module events and assert events are emitted
    subscription_result = await subscribe_events(events=["speculation"])  # Module subscription
    subscription_id = subscription_result["subscription"]
    # Trigger prefetch and collect events
    with wait_for_events(["speculation.prefetchStatusUpdated"]) as waiter:
        await add_speculation_rules_and_link(new_tab, speculation_rules, prefetch_target)
        # Wait for events to be emitted
        events = await waiter.get_events(lambda events: len(events) >= 2)
    # Phase 2: Unsubscribe from module and assert events are NOT emitted
    await bidi_session.session.unsubscribe(subscriptions=[subscription_id])
    # Reload the page to get a clean state
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    # Trigger another prefetch after unsubscribing from module
    prefetch_target_2 = url("/common/square.png", protocol="https")
    speculation_rules_2 = f'''{{
        "prefetch": [{{
            "where": {{ "href_matches": "{prefetch_target_2}" }},
            "eagerness": "immediate"
        }}]
    }}'''
    # Set up waiter but don't expect any events this time
    with wait_for_events(["speculation.prefetchStatusUpdated"]) as waiter:
        await add_speculation_rules_and_link(new_tab, speculation_rules_2, prefetch_target_2)
        with pytest.raises(TimeoutException):
            await waiter.get_events(lambda events: len(events) >= 1, timeout=0.5)
@pytest.mark.asyncio
async def test_unsubscribe_from_prefetch_status_updated(
    bidi_session
):
    '''Test unsubscribing from prefetch status updated events.'''
    # Subscribe to prefetch status updated events
    subscription_result = await bidi_session.session.subscribe(
        events=["speculation.prefetchStatusUpdated"]
    )
    subscription_id = subscription_result["subscription"]
    # Unsubscribe immediately
    await bidi_session.session.unsubscribe(subscriptions=[subscription_id])