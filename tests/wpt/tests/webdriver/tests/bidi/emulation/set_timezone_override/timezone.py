import pytest
import pytest_asyncio
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


LOG_ENTRY_ADDED = "log.entryAdded"
GET_TIMEZONE_SCRIPT = "Intl.DateTimeFormat().resolvedOptions().timeZone"


def shared_worker_page(worker_url):
    return f"""<script>
        window.worker = new SharedWorker('{worker_url}');
        window.worker.port.onmessage = (event) => {{
            console.log(event.data);
        }};
    </script>"""


def service_worker_page(worker_url, extra_script=""):
    return f"""<script>
        navigator.serviceWorker.addEventListener('message', (event) => {{
            console.log(event.data);
        }});
        window.onRegistration =
          navigator.serviceWorker.register('{worker_url}');
        navigator.serviceWorker.startMessages();
        {extra_script}
    </script>"""


@pytest_asyncio.fixture
async def get_log_entry_with_worker_message(
    bidi_session,
    wait_for_event,
    wait_for_future_safe,
):
    async def get_log_entry_with_worker_message(context, trigger):
        on_entry_added = wait_for_event(LOG_ENTRY_ADDED)
        await bidi_session.script.call_function(
            function_declaration=f"()=>{trigger}",
            arguments=[],
            target=ContextTarget(context),
            await_promise=False,
        )
        return await wait_for_future_safe(on_entry_added)

    return get_log_entry_with_worker_message


async def test_timezone_set_override_and_reset(bidi_session, top_context,
        get_current_timezone, default_timezone, some_timezone,
        another_timezone):
    assert await get_current_timezone(top_context) == default_timezone

    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[top_context["context"]],
        timezone=some_timezone
    )

    assert await get_current_timezone(top_context) == some_timezone

    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[top_context["context"]],
        timezone=another_timezone
    )

    assert await get_current_timezone(top_context) == another_timezone

    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[top_context["context"]],
        timezone=None
    )

    assert await get_current_timezone(top_context) == default_timezone


@pytest.mark.parametrize("timezone_offset", ["+10", "-10"])
async def test_timezone_offset(
    bidi_session, new_tab, get_timezone_offset, timezone_offset
):
    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"]], timezone=f"{timezone_offset}:00"
    )

    assert await get_timezone_offset(1753453789196, new_tab) == int(
        timezone_offset
    ) * (-60)


async def test_new_dedicated_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    inline,
    get_current_timezone,
    default_timezone,
    some_timezone,
    wait_for_event,
    wait_for_future_safe
):
    assert await get_current_timezone(new_tab) == default_timezone

    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"]],
        timezone=some_timezone
    )
    await subscribe_events([LOG_ENTRY_ADDED])

    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)
    worker_url = inline(f"""postMessage({GET_TIMEZONE_SCRIPT})""", doctype="js")
    url = inline(f"""<script>
        const worker = new Worker('{worker_url}');
        worker.onmessage = (event) => {{
            console.log(event.data);
        }};
    </script>""")
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )
    event = await wait_for_future_safe(on_entry_added)

    assert event["text"] == some_timezone

    # Reset timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"]],
        timezone=None
    )


async def test_existing_dedicated_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    default_timezone,
    some_timezone,
    get_log_entry_with_worker_message,
    inline
):
    await subscribe_events([LOG_ENTRY_ADDED])

    # Create a dedicated worker that logs its current timezone whenever it
    # receives a message.
    worker_url = inline(f"""onmessage = () => {{postMessage({GET_TIMEZONE_SCRIPT});}};""", doctype="js")
    url = inline(f"""<script>
        window.worker = new Worker('{worker_url}');
        window.worker.onmessage = (event) => {{
            console.log(event.data);
        }};
    </script>""")
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    trigger = "window.worker.postMessage('test')"

    # Verify the worker initially reports the default timezone.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == default_timezone

    # Set timezone override on the existing worker.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"]],
        timezone=some_timezone
    )

    # Verify the existing worker now reports the overridden timezone.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == some_timezone

    # Reset timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"]],
        timezone=None
    )

    # Verify the existing worker now reports the original timezone.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == default_timezone


async def test_new_shared_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    some_timezone,
    wait_for_event,
    wait_for_future_safe,
    inline
):
    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"]],
        timezone=some_timezone
    )
    await subscribe_events([LOG_ENTRY_ADDED])

    worker_url = inline(
        f"""self.onconnect = (event) => {{
            const port = event.ports[0];
            port.postMessage({GET_TIMEZONE_SCRIPT});
        }};""",
        doctype="js"
    )

    url = inline(shared_worker_page(worker_url))
    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )
    event = await wait_for_future_safe(on_entry_added)

    assert event["text"] == some_timezone

    # Reset timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"]],
        timezone=None
    )


async def test_existing_shared_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    default_timezone,
    some_timezone,
    inline,
    get_log_entry_with_worker_message,
):
    await subscribe_events([LOG_ENTRY_ADDED])

    # Create a shared worker that logs its current timezone whenever it
    # receives a message.
    worker_url = inline(
        f"""self.onconnect = (event) => {{
            const port = event.ports[0];
            port.onmessage = () => {{
                port.postMessage({GET_TIMEZONE_SCRIPT});
            }};
        }};""",
        doctype="js"
    )

    url = inline(shared_worker_page(worker_url))

    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    trigger = "window.worker.port.postMessage('test')"

    # Verify the worker initially reports the default timezone.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == default_timezone

    # Set timezone override on the existing worker.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"]],
        timezone=some_timezone
    )

    # Verify the existing worker now reports the overridden timezone.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == some_timezone

    # Reset timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"]],
        timezone=None
    )

    # Verify the existing worker now reports the original timezone.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == default_timezone


async def test_new_shared_worker_multiple_contexts(
    bidi_session,
    new_tab,
    subscribe_events,
    some_timezone,
    another_timezone,
    wait_for_event,
    wait_for_future_safe,
    inline,
    get_log_entry_with_worker_message,
):
    # Create a second browsing context that will share the worker with new_tab.
    another_tab = await bidi_session.browsing_context.create(type_hint="tab")

    # Set timezone override to the first context.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"]],
        timezone=some_timezone
    )
    await subscribe_events([LOG_ENTRY_ADDED])

    worker_url = inline(
        f"""self.onconnect = (event) => {{
            const port = event.ports[0];
            port.onmessage = () => {{
                port.postMessage({GET_TIMEZONE_SCRIPT});
            }};
            port.postMessage({GET_TIMEZONE_SCRIPT});
        }};""",
        doctype="js"
    )

    url = inline(shared_worker_page(worker_url))

    # First context creates the shared worker.
    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )
    event = await wait_for_future_safe(on_entry_added)
    assert event["text"] == some_timezone

    # Second context connects to the same shared worker.
    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)
    await bidi_session.browsing_context.navigate(
        url=url, context=another_tab["context"], wait="complete"
    )
    event = await wait_for_future_safe(on_entry_added)
    # Make sure that the override also applies to the worker
    # in the other context.
    assert event["text"] == some_timezone

    # Set timezone override to the second context.
    await bidi_session.emulation.set_timezone_override(
        contexts=[another_tab["context"]],
        timezone=another_timezone
    )

    trigger = "window.worker.port.postMessage('test')"

    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == another_timezone

    event = await get_log_entry_with_worker_message(another_tab["context"], trigger)
    assert event["text"] == another_timezone

    # Reset timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"], another_tab["context"]],
        timezone=None
    )


async def test_new_service_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    get_current_timezone,
    default_timezone,
    some_timezone,
    wait_for_event,
    wait_for_future_safe,
    inline
):
    assert await get_current_timezone(new_tab) == default_timezone

    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[new_tab["context"]],
        timezone=some_timezone
    )
    await subscribe_events([LOG_ENTRY_ADDED])

    worker_url = inline(
        f"""self.addEventListener('activate', (event) => {{
            event.waitUntil((async () => {{
                await self.clients.claim();
                const clients = await self.clients.matchAll({{includeUncontrolled: true}});
                for (const client of clients) {{
                    client.postMessage({GET_TIMEZONE_SCRIPT});
                }}
            }})());
        }});""",
        doctype="js"
    )
    url = inline(service_worker_page(worker_url))

    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    event = await wait_for_future_safe(on_entry_added)
    try:
        assert event["text"] == some_timezone

        # Reset timezone override.
        await bidi_session.emulation.set_timezone_override(
            contexts=[new_tab["context"]],
            timezone=None
        )
    finally:
        # Unregister the service worker registration.
        await bidi_session.script.evaluate(
            expression="""window.onRegistration.then(r => r.unregister())""",
            await_promise=True,
            target=ContextTarget(new_tab["context"]),
        )


async def test_existing_service_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    default_timezone,
    some_timezone,
    inline,
    get_log_entry_with_worker_message,
):
    await subscribe_events([LOG_ENTRY_ADDED])

    # Create a service worker that logs its current timezone whenever it
    # receives a message.
    worker_url = inline(
        f"""self.addEventListener('message', (event) => {{
            event.source.postMessage({GET_TIMEZONE_SCRIPT});
        }});""",
        doctype="js"
    )
    extra_script = """window.sendMessage = async () => {
            const registration = await window.onRegistration;
            await navigator.serviceWorker.ready;
            registration.active.postMessage('test');
        };"""
    url = inline(service_worker_page(worker_url, extra_script=extra_script))

    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    trigger = "window.sendMessage()"

    try:
        # Verify the worker initially reports the default timezone.
        event = await get_log_entry_with_worker_message(
            new_tab["context"], trigger
        )
        assert event["text"] == default_timezone

        # Set timezone override on the existing worker.
        await bidi_session.emulation.set_timezone_override(
            contexts=[new_tab["context"]],
            timezone=some_timezone
        )

        # Verify the existing worker now reports the overridden timezone.
        event = await get_log_entry_with_worker_message(
            new_tab["context"], trigger
        )
        assert event["text"] == some_timezone

        # Reset timezone override.
        await bidi_session.emulation.set_timezone_override(
            contexts=[new_tab["context"]],
            timezone=None
        )

        # Verify the existing worker now reports the original timezone.
        event = await get_log_entry_with_worker_message(
            new_tab["context"], trigger
        )
        assert event["text"] == default_timezone
    finally:
        # Unregister the service worker registration.
        await bidi_session.script.evaluate(
            expression="""window.onRegistration.then(r => r.unregister())""",
            await_promise=True,
            target=ContextTarget(new_tab["context"]),
        )
