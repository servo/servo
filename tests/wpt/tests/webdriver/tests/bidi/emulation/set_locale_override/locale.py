import pytest
import pytest_asyncio
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


LOG_ENTRY_ADDED = "log.entryAdded"
GET_NAVIGATOR_LANGUAGE_SCRIPT = "navigator.language"
GET_LOCALE_SCRIPT = "new Intl.DateTimeFormat().resolvedOptions().locale"


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


def accept_language_request(echo_url, request_type):
    """Return a JS Promise expression that resolves to the `Accept-Language`
    header value the header-echo handler at `echo_url` received for a request
    made via `request_type` (either "fetch" or "xhr"). Caching is bypassed so
    that repeated requests always hit the network and observe the current
    locale override."""
    if request_type == "fetch":
        return f"""fetch("{echo_url}", {{cache: "no-store"}})
            .then((response) => response.json())
            .then((data) => data.headers["accept-language"][0])"""
    return f"""new Promise((resolve, reject) => {{
        const xhr = new XMLHttpRequest();
        xhr.open("GET", "{echo_url}?nocache=" + Math.random());
        xhr.responseType = "json";
        xhr.onload = () => resolve(xhr.response.headers["accept-language"][0]);
        xhr.onerror = () => reject(new Error("XHR request failed"));
        xhr.send();
    }})"""


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


async def test_locale_set_override_and_reset(
    bidi_session,
    new_tab,
    another_locale,
    assert_locale_against_default,
    assert_locale_against_value,
    some_locale,
):
    await assert_locale_against_default(new_tab)

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=some_locale
    )

    await assert_locale_against_value(some_locale, new_tab)

    # Set another locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=another_locale
    )

    await assert_locale_against_value(another_locale, new_tab)

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=None
    )

    await assert_locale_against_default(new_tab)


@pytest.mark.parametrize(
    "value",
    [
        # Simple language code (2-letter).
        "en",
        # Language and region (both 2-letter).
        "en-US",
        # Language and script (4-letter).
        "sr-Latn",
        # Language, script, and region.
        "zh-Hans-CN",
    ],
)
async def test_locale_values(
    bidi_session,
    new_tab,
    assert_locale_against_default,
    assert_locale_against_value,
    value,
):
    await assert_locale_against_default(new_tab)

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=value
    )

    await assert_locale_against_value(value, new_tab)


@pytest.mark.parametrize(
    "locale,expected_locale",
    [
        # Locale with Unicode extension keyword for collation.
        ("de-DE-u-co-phonebk", "de-DE"),
        # Lowercase language and region.
        ("fr-ca", "fr-CA"),
        # Uppercase language and region (should be normalized by Intl.Locale).
        ("FR-CA", "fr-CA"),
        # Mixed case language and region (should be normalized by Intl.Locale).
        ("fR-cA", "fr-CA"),
        # Locale with transform extension (simple case).
        ("en-t-zh", "en"),
    ],
)
async def test_locale_values_normalized_by_intl(
    bidi_session,
    new_tab,
    get_current_locale,
    default_locale,
    locale,
    expected_locale,
):
    assert await get_current_locale(new_tab) == default_locale

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=locale
    )

    assert await get_current_locale(new_tab) == expected_locale


@pytest.mark.parametrize(
    "worker_script",
    [
        GET_NAVIGATOR_LANGUAGE_SCRIPT,
        GET_LOCALE_SCRIPT
    ],
)
async def test_new_dedicated_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    inline,
    some_locale,
    wait_for_event,
    wait_for_future_safe,
    worker_script
):
    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=some_locale
    )
    await subscribe_events([LOG_ENTRY_ADDED])

    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)
    worker_url = inline(f"""postMessage({worker_script})""", doctype="js")

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

    assert event["text"] == some_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=None
    )


@pytest.mark.parametrize(
    "worker_script, type",
    [
        (GET_NAVIGATOR_LANGUAGE_SCRIPT, "navigator"),
        (GET_LOCALE_SCRIPT, "locale")
    ],
)
async def test_existing_dedicated_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    default_locale,
    default_navigator_language,
    some_locale,
    get_log_entry_with_worker_message,
    worker_script,
    inline,
    type
):
    await subscribe_events([LOG_ENTRY_ADDED])

    # Create a dedicated worker that logs its current locale whenever it
    # receives a message.
    worker_url = inline(f"""onmessage = () => {{postMessage({worker_script});}};""", doctype="js")

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

    # Verify the worker initially reports the default locale.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    default_locale_value = default_navigator_language if type == "navigator" else default_locale
    assert event["text"] == default_locale_value

    # Set locale override on the existing worker.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]],
        locale=some_locale
    )

    # Verify the existing worker now reports the overridden locale.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == some_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]],
        locale=None
    )

    # Verify the existing worker now reports the original locale.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == default_locale_value


@pytest.mark.parametrize(
    "worker_script",
    [
        GET_NAVIGATOR_LANGUAGE_SCRIPT,
        GET_LOCALE_SCRIPT
    ],
)
async def test_new_shared_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    some_locale,
    wait_for_event,
    wait_for_future_safe,
    inline,
    worker_script
):
    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=some_locale
    )
    await subscribe_events([LOG_ENTRY_ADDED])

    worker_url = inline(
        f"""self.onconnect = (event) => {{
            const port = event.ports[0];
            port.postMessage({worker_script});
        }};""",
        doctype="js"
    )

    url = inline(shared_worker_page(worker_url))
    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )
    event = await wait_for_future_safe(on_entry_added)

    assert event["text"] == some_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=None
    )


@pytest.mark.parametrize(
    "worker_script, type",
    [
        (GET_NAVIGATOR_LANGUAGE_SCRIPT, "navigator"),
        (GET_LOCALE_SCRIPT, "locale")
    ],
)
async def test_existing_shared_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    default_navigator_language,
    default_locale,
    some_locale,
    inline,
    get_log_entry_with_worker_message,
    worker_script,
    type
):
    await subscribe_events([LOG_ENTRY_ADDED])

    # Create a shared worker that logs its current locale whenever it
    # receives a message.
    worker_url = inline(
        f"""self.onconnect = (event) => {{
            const port = event.ports[0];
            port.onmessage = () => {{
                port.postMessage({worker_script});
            }};
        }};""",
        doctype="js"
    )

    url = inline(shared_worker_page(worker_url))

    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    trigger = "window.worker.port.postMessage('test')"

    # Verify the worker initially reports the default locale.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    default_locale_value = default_navigator_language if type == "navigator" else default_locale
    assert event["text"] == default_locale_value

    # Set locale override on the existing worker.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]],
        locale=some_locale
    )

    # Verify the existing worker now reports the overridden locale.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == some_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]],
        locale=None
    )

    # Verify the existing worker now reports the original locale.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == default_locale_value


@pytest.mark.parametrize(
    "worker_script",
    [
        GET_NAVIGATOR_LANGUAGE_SCRIPT,
        GET_LOCALE_SCRIPT
    ],
)
async def test_new_shared_worker_multiple_contexts(
    bidi_session,
    new_tab,
    subscribe_events,
    some_locale,
    another_locale,
    wait_for_event,
    wait_for_future_safe,
    inline,
    get_log_entry_with_worker_message,
    worker_script
):
    # Create a second browsing context that will share the worker with new_tab.
    another_tab = await bidi_session.browsing_context.create(type_hint="tab")

    # Set locale override to the first context.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]],
        locale=some_locale
    )
    await subscribe_events([LOG_ENTRY_ADDED])

    worker_url = inline(
        f"""self.onconnect = (event) => {{
            const port = event.ports[0];
            port.onmessage = () => {{
                port.postMessage({worker_script});
            }};
            port.postMessage({worker_script});
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
    assert event["text"] == some_locale

    # Second context connects to the same shared worker.
    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)
    await bidi_session.browsing_context.navigate(
        url=url, context=another_tab["context"], wait="complete"
    )
    event = await wait_for_future_safe(on_entry_added)
    # Make sure that the override also applies to the worker
    # in the other context.
    assert event["text"] == some_locale

    # Set locale override to the second context.
    await bidi_session.emulation.set_locale_override(
        contexts=[another_tab["context"]],
        locale=another_locale
    )

    trigger = "window.worker.port.postMessage('test')"

    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == another_locale

    event = await get_log_entry_with_worker_message(another_tab["context"], trigger)
    assert event["text"] == another_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"], another_tab["context"]],
        locale=None
    )


@pytest.mark.parametrize(
    "worker_script",
    [
        GET_NAVIGATOR_LANGUAGE_SCRIPT,
        GET_LOCALE_SCRIPT
    ],
    ids=["navigator", "locale"]
)
async def test_new_service_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    some_locale,
    wait_for_event,
    wait_for_future_safe,
    inline,
    worker_script
):
    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]],
        locale=some_locale
    )
    await subscribe_events([LOG_ENTRY_ADDED])

    worker_url = inline(
        f"""self.addEventListener('activate', (event) => {{
            event.waitUntil((async () => {{
                await self.clients.claim();
                const clients = await self.clients.matchAll({{includeUncontrolled: true}});
                for (const client of clients) {{
                    client.postMessage({worker_script});
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
        assert event["text"] == some_locale

        # Reset locale override.
        await bidi_session.emulation.set_locale_override(
            contexts=[new_tab["context"]],
            locale=None
        )
    finally:
        # Unregister the service worker registration.
        await bidi_session.script.evaluate(
            expression="""window.onRegistration.then(r => r.unregister())""",
            await_promise=True,
            target=ContextTarget(new_tab["context"]),
        )


@pytest.mark.parametrize(
    "worker_script, type",
    [
        (GET_NAVIGATOR_LANGUAGE_SCRIPT, "navigator"),
        (GET_LOCALE_SCRIPT, "locale")
    ],
    ids=["navigator", "locale"]
)
async def test_existing_service_worker(
    bidi_session,
    new_tab,
    subscribe_events,
    default_navigator_language,
    default_locale,
    some_locale,
    inline,
    get_log_entry_with_worker_message,
    worker_script,
    type
):
    await subscribe_events([LOG_ENTRY_ADDED])

    # Create a service worker that logs its current locale whenever it
    # receives a message.
    worker_url = inline(
        f"""self.addEventListener('message', (event) => {{
            event.source.postMessage({worker_script});
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
        # Verify the worker initially reports the default locale.
        event = await get_log_entry_with_worker_message(
            new_tab["context"], trigger
        )
        default_locale_value = default_navigator_language if type == "navigator" else default_locale
        assert event["text"] == default_locale_value

        # Set locale override on the existing worker.
        await bidi_session.emulation.set_locale_override(
            contexts=[new_tab["context"]],
            locale=some_locale
        )

        # Verify the existing worker now reports the overridden locale.
        event = await get_log_entry_with_worker_message(
            new_tab["context"], trigger
        )
        assert event["text"] == some_locale

        # Reset locale override.
        await bidi_session.emulation.set_locale_override(
            contexts=[new_tab["context"]],
            locale=None
        )

        # Verify the existing worker now reports the original locale.
        event = await get_log_entry_with_worker_message(
            new_tab["context"], trigger
        )
        assert event["text"] == default_locale_value
    finally:
        # Unregister the service worker registration.
        await bidi_session.script.evaluate(
            expression="""window.onRegistration.then(r => r.unregister())""",
            await_promise=True,
            target=ContextTarget(new_tab["context"]),
        )


@pytest.mark.parametrize("request_type", ["fetch", "xhr"])
async def test_new_dedicated_worker_accept_language(
    bidi_session,
    new_tab,
    subscribe_events,
    inline,
    url,
    some_locale,
    wait_for_event,
    wait_for_future_safe,
    request_type,
):
    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=some_locale
    )
    await subscribe_events([LOG_ENTRY_ADDED])

    on_entry_added = wait_for_event(LOG_ENTRY_ADDED)
    echo_url = url("webdriver/tests/support/http_handlers/headers_echo.py")
    request = accept_language_request(echo_url, request_type)

    # Create a dedicated worker that requests the header-echo handler and logs
    # the Accept-Language header the server received.
    worker_url = inline(
        f"""({request}).then((acceptLanguage) => postMessage(acceptLanguage));""",
        doctype="js",
    )
    page_url = inline(f"""<script>
        const worker = new Worker("{worker_url}");
        worker.onmessage = (event) => {{
            console.log(event.data);
        }};
    </script>""")
    await bidi_session.browsing_context.navigate(
        url=page_url, context=new_tab["context"], wait="complete"
    )
    event = await wait_for_future_safe(on_entry_added)

    # Verify the worker-initiated request used the overridden Accept-Language.
    assert event["text"] == some_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=None
    )


@pytest.mark.parametrize("request_type", ["fetch", "xhr"])
async def test_existing_dedicated_worker_accept_language(
    bidi_session,
    new_tab,
    subscribe_events,
    inline,
    url,
    default_accept_language,
    some_locale,
    get_log_entry_with_worker_message,
    request_type,
):
    await subscribe_events([LOG_ENTRY_ADDED])

    echo_url = url("webdriver/tests/support/http_handlers/headers_echo.py")
    request = accept_language_request(echo_url, request_type)

    # Create a dedicated worker that requests the header-echo handler and logs
    # the Accept-Language header the server received whenever it receives a
    # message.
    worker_url = inline(
        f"""onmessage = () => {{
            ({request}).then((acceptLanguage) => postMessage(acceptLanguage));
        }};""",
        doctype="js",
    )
    page_url = inline(f"""<script>
        window.worker = new Worker("{worker_url}");
        window.worker.onmessage = (event) => {{
            console.log(event.data);
        }};
    </script>""")
    await bidi_session.browsing_context.navigate(
        url=page_url, context=new_tab["context"], wait="complete"
    )

    trigger = "window.worker.postMessage('test')"

    # Verify the existing worker initially uses the default Accept-Language.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == default_accept_language

    # Set locale override on the existing worker.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=some_locale
    )

    # Verify the existing worker now uses the overridden Accept-Language.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == some_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=None
    )

    # Verify the existing worker now uses the original Accept-Language.
    event = await get_log_entry_with_worker_message(new_tab["context"], trigger)
    assert event["text"] == default_accept_language
