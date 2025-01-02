import asyncio
import base64
import copy
import json
from typing import Any, Coroutine, Mapping
from urllib.parse import urlunsplit

import pytest
import pytest_asyncio

from tests.support.asserts import assert_pdf
from tests.support.image import cm_to_px, png_dimensions, ImageDifference
from tests.support.sync import AsyncPoll
from webdriver.bidi.error import (
    InvalidArgumentException,
    NoSuchFrameException,
    NoSuchInterceptException,
    NoSuchRequestException,
    NoSuchScriptException,
    NoSuchUserContextException,
    UnableToSetCookieException,
    UnderspecifiedStoragePartitionException
)
from webdriver.bidi.modules.input import Actions
from webdriver.bidi.modules.script import ContextTarget
from webdriver.error import TimeoutException


@pytest_asyncio.fixture
async def add_preload_script(bidi_session):
    preload_scripts_ids = []

    async def add_preload_script(function_declaration, arguments=None, contexts=None, sandbox=None):
        script = await bidi_session.script.add_preload_script(
            function_declaration=function_declaration,
            arguments=arguments,
            contexts=contexts,
            sandbox=sandbox,
        )
        preload_scripts_ids.append(script)

        return script

    yield add_preload_script

    for script in reversed(preload_scripts_ids):
        try:
            await bidi_session.script.remove_preload_script(script=script)
        except (InvalidArgumentException, NoSuchScriptException):
            pass


@pytest_asyncio.fixture
async def execute_as_async(bidi_session):
    async def execute_as_async(sync_func, **kwargs):
        # Ideally we should use asyncio.to_thread() but it's not available in
        # Python 3.8 which wpt tests have to support.
        return await bidi_session.event_loop.run_in_executor(None, sync_func, **kwargs)

    return execute_as_async


@pytest_asyncio.fixture
async def subscribe_events(bidi_session):
    subscriptions = []

    async def subscribe_events(events, contexts=None):
        await bidi_session.session.subscribe(events=events, contexts=contexts)
        subscriptions.append((events, contexts))

    yield subscribe_events

    for events, contexts in reversed(subscriptions):
        try:
            await bidi_session.session.unsubscribe(events=events,
                                                   contexts=contexts)
        except (InvalidArgumentException, NoSuchFrameException):
            pass


@pytest_asyncio.fixture
async def set_cookie(bidi_session):
    """
    Set a cookie and remove them after the test is finished.
    """
    cookies = []

    async def set_cookie(cookie, partition=None):
        partition_descriptor = None
        set_cookie_result = await bidi_session.storage.set_cookie(cookie=cookie, partition=partition)
        if set_cookie_result["partitionKey"] != {}:
            # Make a copy of the partition key, as the original dict is used for assertion.
            partition_descriptor = copy.deepcopy(set_cookie_result["partitionKey"])
            partition_descriptor["type"] = "storageKey"
        # Store the cookie partition to remove the cookie after the test.
        # The requested partition can be a browsing context, so the returned partition descriptor (it's always of type
        # "storageKey") is used.
        cookies.append((copy.deepcopy(cookie), partition_descriptor))
        return set_cookie_result

    yield set_cookie

    for cookie, partition in reversed(cookies):
        try:
            await bidi_session.storage.delete_cookies(filter=cookie, partition=partition)
        except (InvalidArgumentException, UnableToSetCookieException, UnderspecifiedStoragePartitionException):
            pass


@pytest_asyncio.fixture
async def new_tab(bidi_session):
    """Open and focus a new tab to run the test in a foreground tab."""
    new_tab = await bidi_session.browsing_context.create(type_hint='tab')

    yield new_tab

    try:
        await bidi_session.browsing_context.close(context=new_tab["context"])
    except NoSuchFrameException:
        print(f"Tab with id {new_tab['context']} has already been closed")


@pytest.fixture
def send_blocking_command(bidi_session):
    """Send a blocking command that awaits until the BiDi response has been received."""
    async def send_blocking_command(command: str, params: Mapping[str, Any]) -> Mapping[str, Any]:
        future_response = await bidi_session.send_command(command, params)
        return await future_response
    return send_blocking_command


@pytest.fixture
def wait_for_event(bidi_session, event_loop):
    """Wait until the BiDi session emits an event and resolve the event data."""
    remove_listeners = []

    def wait_for_event(event_name: str):
        future = event_loop.create_future()

        async def on_event(_, data):
            remove_listener()
            remove_listeners.remove(remove_listener)
            future.set_result(data)

        remove_listener = bidi_session.add_event_listener(event_name, on_event)
        remove_listeners.append(remove_listener)
        return future

    yield wait_for_event

    # Cleanup any leftover callback for which no event was captured.
    for remove_listener in remove_listeners:
        remove_listener()


@pytest.fixture
def wait_for_future_safe(configuration):
    """Wait for the given future for a given amount of time.
    Fails gracefully if the future does not resolve within the given timeout."""

    async def wait_for_future_safe(future: Coroutine, timeout: float = 2.0):
        try:
            return await asyncio.wait_for(
                asyncio.shield(future),
                timeout=timeout * configuration["timeout_multiplier"],
            )
        except asyncio.TimeoutError:
            raise TimeoutException("Future did not resolve within the given timeout")

    return wait_for_future_safe


@pytest.fixture
def current_time(bidi_session, top_context):
    """Get the current time stamp in ms from the remote end.

    This is required especially when tests are run on different devices like
    for Android, where it's not guaranteed that both machines are in sync.
    """
    async def current_time():
        result = await bidi_session.script.evaluate(
            expression="Date.now()",
            target=ContextTarget(top_context["context"]),
            await_promise=True)
        return result["value"]

    return current_time


@pytest.fixture
def add_and_remove_iframe(bidi_session):
    """Create a frame, wait for load, and remove it.

    Return the frame's context id, which allows to test for invalid
    browsing context references.
    """

    async def closed_frame(context):
        initial_contexts = await bidi_session.browsing_context.get_tree(root=context["context"])
        resp = await bidi_session.script.call_function(
            function_declaration="""(url) => {
                const iframe = document.createElement("iframe");
                // Once we're confident implementations support returning the iframe, just
                // return that directly. For now generate a unique id to use as a handle.
                const id = `testframe-${Math.random()}`;
                iframe.id = id;
                iframe.src = url;
                document.documentElement.lastElementChild.append(iframe);
                return new Promise(resolve => iframe.onload = () => resolve(id));
            }""",
            target=ContextTarget(context["context"]),
            await_promise=True)
        iframe_dom_id = resp["value"]

        new_contexts = await bidi_session.browsing_context.get_tree(root=context["context"])
        added_contexts = ({item["context"] for item in new_contexts[0]["children"]} -
                          {item["context"] for item in initial_contexts[0]["children"]})
        assert len(added_contexts) == 1
        frame_id = added_contexts.pop()

        await bidi_session.script.evaluate(
            expression=f"document.getElementById('{iframe_dom_id}').remove()",
            target=ContextTarget(context["context"]),
            await_promise=False)

        return frame_id
    return closed_frame


@pytest.fixture
def load_pdf_bidi(bidi_session, test_page_with_pdf_js, top_context):
    """Load a PDF document in the browser using pdf.js"""
    async def load_pdf_bidi(encoded_pdf_data, context=top_context["context"]):
        url = test_page_with_pdf_js(encoded_pdf_data)

        await bidi_session.browsing_context.navigate(
            context=context, url=url, wait="complete"
        )

    return load_pdf_bidi


@pytest.fixture
def get_pdf_content(bidi_session, top_context, load_pdf_bidi):
    """Load a PDF document in the browser using pdf.js and extract content from the document"""
    async def get_pdf_content(encoded_pdf_data, context=top_context["context"]):
        await load_pdf_bidi(encoded_pdf_data=encoded_pdf_data, context=context)

        result = await bidi_session.script.call_function(
            function_declaration="() => { return window.getText(); }",
            target=ContextTarget(context),
            await_promise=True,
        )

        return result

    return get_pdf_content


@pytest.fixture
def assert_pdf_content(new_tab, get_pdf_content):
    """Assert PDF with provided content"""
    async def assert_pdf_content(pdf, expected_content):
        assert_pdf(pdf)

        pdf_content = await get_pdf_content(pdf, new_tab["context"])

        assert pdf_content == {
            "type": "array",
            "value": expected_content,
        }

    return assert_pdf_content


@pytest.fixture
def assert_pdf_dimensions(render_pdf_to_png_bidi):
    """Assert PDF dimensions"""
    async def assert_pdf_dimensions(pdf, expected_dimensions):
        assert_pdf(pdf)

        png = await render_pdf_to_png_bidi(pdf)
        width, height = png_dimensions(png)

        # account for potential rounding errors
        assert (height - 1) <= cm_to_px(expected_dimensions["height"]) <= (height + 1)
        assert (width - 1) <= cm_to_px(expected_dimensions["width"]) <= (width + 1)

    return assert_pdf_dimensions


@pytest.fixture
def assert_pdf_image(
    get_reference_png, render_pdf_to_png_bidi, compare_png_bidi
):
    """Assert PDF with image generated for provided html"""
    async def assert_pdf_image(pdf, reference_html, expected):
        assert_pdf(pdf)

        reference_png = await get_reference_png(reference_html)
        page_without_background_png = await render_pdf_to_png_bidi(pdf)
        comparison_without_background = await compare_png_bidi(
            reference_png,
            page_without_background_png,
        )

        assert comparison_without_background.equal() == expected

    return assert_pdf_image


@pytest.fixture
def compare_png_bidi(bidi_session, url):
    async def compare_png_bidi(img1, img2):
        """Calculate difference statistics between two PNG images.

        :param img1: Bytes of first PNG image
        :param img2: Bytes of second PNG image
        :returns: ImageDifference representing the total number of different pixels,
                and maximum per-channel difference between the images.
        """
        if img1 == img2:
            return ImageDifference(0, 0)

        width, height = png_dimensions(img1)
        assert (width, height) == png_dimensions(img2)

        context = await bidi_session.browsing_context.create(type_hint="tab")
        await bidi_session.browsing_context.navigate(
            context=context["context"],
            url=url("/webdriver/tests/support/html/render.html"),
            wait="complete",
        )
        result = await bidi_session.script.call_function(
            function_declaration="""(img1, img2, width, height) => {
            return compare(img1, img2, width, height)
            }""",
            target=ContextTarget(context["context"]),
            arguments=[
                {"type": "string", "value": base64.encodebytes(img1).decode()},
                {"type": "string", "value": base64.encodebytes(img2).decode()},
                {"type": "number", "value": width},
                {"type": "number", "value": height},
            ],
            await_promise=True,
        )
        await bidi_session.browsing_context.close(context=context["context"])
        assert result["type"] == "object"
        assert set(item[0] for item in result["value"]) == {"totalPixels", "maxDifference"}
        for item in result["value"]:
            assert len(item) == 2
            assert item[1]["type"] == "number"
            if item[0] == "totalPixels":
                total_pixels = item[1]["value"]
            elif item[0] == "maxDifference":
                max_difference = item[1]["value"]
            else:
                raise Exception(f"Unexpected object key ${item[0]}")
        return ImageDifference(total_pixels, max_difference)
    return compare_png_bidi


@pytest.fixture
def current_url(bidi_session):
    async def current_url(context):
        contexts = await bidi_session.browsing_context.get_tree(root=context, max_depth=0)
        return contexts[0]["url"]

    return current_url


@pytest.fixture
def get_element(bidi_session, top_context):
    async def get_element(css_selector, context=top_context):
        result = await bidi_session.script.evaluate(
            expression=f"document.querySelector('{css_selector}')",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )
        return result
    return get_element


@pytest.fixture
def get_reference_png(
    bidi_session, inline, render_pdf_to_png_bidi, top_context
):
    """Print to PDF provided content and render it to png"""
    async def get_reference_png(reference_content, context=top_context["context"]):
        reference_page = inline(reference_content)
        await bidi_session.browsing_context.navigate(
            context=context, url=reference_page, wait="complete"
        )

        reference_pdf = await bidi_session.browsing_context.print(
            context=context,
            background=True,
        )

        return await render_pdf_to_png_bidi(reference_pdf)

    return get_reference_png


@pytest.fixture
def render_pdf_to_png_bidi(bidi_session, new_tab, url):
    """Render a PDF document to png"""

    async def render_pdf_to_png_bidi(
        encoded_pdf_data, page=1
    ):
        await bidi_session.browsing_context.navigate(
            context=new_tab["context"],
            url=url(path="/print_pdf_runner.html"),
            wait="complete",
        )

        result = await bidi_session.script.call_function(
            function_declaration=f"""() => {{ return window.render("{encoded_pdf_data}"); }}""",
            target=ContextTarget(new_tab["context"]),
            await_promise=True,
        )
        value = result["value"]
        index = page - 1

        assert 0 <= index < len(value)

        image_string = value[index]["value"]
        image_string_without_data_type = image_string[image_string.find(",") +
                                                      1:]

        return base64.b64decode(image_string_without_data_type)

    return render_pdf_to_png_bidi


@pytest.fixture
def load_static_test_page(bidi_session, url, top_context):
    """Navigate to a test page from the support/html folder."""

    async def load_static_test_page(page, context=top_context):
        await bidi_session.browsing_context.navigate(
            context=context["context"],
            url=url(f"/webdriver/tests/support/html/{page}"),
            wait="complete",
        )

    return load_static_test_page


@pytest_asyncio.fixture
async def create_user_context(bidi_session):
    """Create a user context and ensure it is removed at the end of the test."""

    user_contexts = []

    async def create_user_context():
        nonlocal user_contexts
        user_context = await bidi_session.browser.create_user_context()
        user_contexts.append(user_context)

        return user_context

    yield create_user_context

    # Remove all created user contexts at the end of the test
    for user_context in user_contexts:
        try:
            await bidi_session.browser.remove_user_context(user_context=user_context)
        except NoSuchUserContextException:
            # Ignore exceptions in case a specific user context was already
            # removed during the test.
            pass


@pytest_asyncio.fixture
async def add_cookie(bidi_session):
    """
    Add a cookie with `document.cookie` and remove them after the test is finished.
    """
    cookies = []

    async def add_cookie(
        context,
        name,
        value,
        domain=None,
        expiry=None,
        path=None,
        same_site="none",
        secure=False,
    ):
        cookie_string = f"{name}={value}"
        cookie = {"name": name, "context": context}

        if domain is not None:
            cookie_string += f";domain={domain}"

        if expiry is not None:
            cookie_string += f";expires={expiry}"

        if path is not None:
            cookie_string += f";path={path}"
            cookie["path"] = path

        if same_site != "none":
            cookie_string += f";SameSite={same_site}"

        if secure is True:
            cookie_string += ";Secure"

        await bidi_session.script.evaluate(
            expression=f"document.cookie = '{cookie_string}'",
            target=ContextTarget(context),
            await_promise=True,
        )

        cookies.append(cookie)

    yield add_cookie

    for cookie in reversed(cookies):
        cookie_string = f"""{cookie["name"]}="""

        if "path" in cookie:
            cookie_string += f""";path={cookie["path"]}"""

        await bidi_session.script.evaluate(
            expression=f"""document.cookie = '{cookie_string};Max-Age=0'""",
            target=ContextTarget(cookie["context"]),
            await_promise=True,
        )


@pytest.fixture
def domain_value(server_config):
    def domain_value(domain="", subdomain=""):
        return server_config["domains"][domain][subdomain]

    return domain_value


@pytest.fixture
def fetch(bidi_session, top_context, configuration):
    """Perform a fetch from the page of the provided context, default to the
    top context.
    """

    async def fetch(
        url,
        method="GET",
        headers=None,
        post_data=None,
        context=top_context,
        timeout_in_seconds=3,
    ):
        method_arg = f"method: '{method}',"

        headers_arg = ""
        if headers is not None:
            headers_arg = f"headers: {json.dumps(headers)},"

        body_arg = ""
        if post_data is not None:
            body_arg = f"body: {json.dumps(post_data)},"

        timeout_in_seconds = timeout_in_seconds * configuration["timeout_multiplier"]
        # Wait for fetch() to resolve a response and for response.text() to
        # resolve as well to make sure the request/response is completed when
        # the helper returns.
        await bidi_session.script.evaluate(
            expression=f"""
                 {{
                   const controller = new AbortController();
                   setTimeout(() => controller.abort(), {timeout_in_seconds * 1000});
                   fetch("{url}", {{
                     {method_arg}
                     {headers_arg}
                     {body_arg}
                     signal: controller.signal,
                   }}).then(response => response.text());
                 }}""",
            target=ContextTarget(context["context"]),
            await_promise=True,
        )

    return fetch


@pytest_asyncio.fixture
async def setup_beforeunload_page(bidi_session, url):
    async def setup_beforeunload_page(context):
        page_url = url("/webdriver/tests/support/html/beforeunload.html")
        await bidi_session.browsing_context.navigate(
            context=context["context"],
            url=page_url,
            wait="complete"
        )

        # Focus the input
        await bidi_session.script.evaluate(
            expression="""
                const input = document.querySelector("input");
                input.focus();
            """,
            target=ContextTarget(context["context"]),
            await_promise=False,
        )

        actions = Actions()
        actions.add_key().send_keys("foo")
        await bidi_session.input.perform_actions(
            actions=actions, context=context["context"]
        )

        return page_url

    return setup_beforeunload_page


@pytest_asyncio.fixture
async def setup_network_test(
    bidi_session,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    top_context,
    url,
):
    """Navigate the provided top level context to the provided url and subscribe
    to network events for the provided set of contexts.

    By default, the test context is top_context["context"], test_url is
    empty.html and contexts is None (meaning we will subscribe to all contexts).

    Returns an `events` dictionary in which the captured network events will be added.
    The keys of the dictionary are network event names (eg. "network.beforeRequestSent"),
    and the value is an array of collected events.
    """
    listeners = []

    async def _setup_network_test(
        events,
        test_url=url("/webdriver/tests/bidi/network/support/empty.html"),
        context=top_context["context"],
        contexts=None,
    ):
        nonlocal listeners

        # Listen for network.responseCompleted for the initial navigation to
        # make sure this event will not be captured unexpectedly by the tests.
        await bidi_session.session.subscribe(
            events=["network.responseCompleted"], contexts=[context]
        )
        on_response_completed = wait_for_event("network.responseCompleted")

        await bidi_session.browsing_context.navigate(
            context=context,
            url=test_url,
            wait="complete",
        )
        await wait_for_future_safe(on_response_completed)
        await bidi_session.session.unsubscribe(
            events=["network.responseCompleted"], contexts=[context]
        )

        await subscribe_events(events, contexts)

        network_events = {}
        for event in events:
            network_events[event] = []

            async def on_event(method, data, event=event):
                network_events[event].append(data)

            listeners.append(bidi_session.add_event_listener(event, on_event))

        return network_events

    yield _setup_network_test

    # cleanup
    for remove_listener in listeners:
        remove_listener()


@pytest_asyncio.fixture
async def add_intercept(bidi_session):
    """Add a network intercept for the provided phases and url patterns, and
    ensure the intercept is removed at the end of the test."""

    intercepts = []

    async def add_intercept(phases, url_patterns, contexts = None):
        nonlocal intercepts
        intercept = await bidi_session.network.add_intercept(
            phases=phases,
            url_patterns=url_patterns,
            contexts=contexts,
        )
        intercepts.append(intercept)

        return intercept

    yield add_intercept

    # Remove all added intercepts at the end of the test
    for intercept in intercepts:
        try:
            await bidi_session.network.remove_intercept(intercept=intercept)
        except NoSuchInterceptException:
            # Ignore exceptions in case a specific intercept was already removed
            # during the test.
            pass


@pytest_asyncio.fixture
async def setup_blocked_request(
    bidi_session,
    setup_network_test,
    url,
    add_intercept,
    fetch,
    wait_for_event,
    wait_for_future_safe,
    top_context,
):
    """Creates an intercept for the provided phase, sends a fetch request that
    should be blocked by this intercept and resolves when the corresponding
    event is received.

    Pass blocked_url to target a specific URL. Otherwise, the test will use
    PAGE_EMPTY_TEXT as default test url.

    Pass navigate=True in order to navigate instead of doing a fetch request.
    If the navigation url should be different from the blocked url, you can
    specify navigate_url.

    For the "authRequired" phase, the request will be sent to the authentication
    http handler. The optional arguments username, password and realm can be used
    to configure the handler.

    Returns the `request` id of the intercepted request.
    """

    # Keep track of blocked requests in order to cancel them with failRequest
    # on test teardown, in case the test did not handle the request.
    blocked_requests = []

    # Blocked auth requests need to resumed using continueWithAuth, they cannot
    # rely on failRequest
    blocked_auth_requests = []

    async def setup_blocked_request(
        phase,
        context=top_context,
        username="user",
        password="password",
        realm="test",
        blocked_url=None,
        navigate=False,
        navigate_url=None,
        **kwargs,
    ):
        await setup_network_test(events=[f"network.{phase}"])

        if blocked_url is None:
            if phase == "authRequired":
                blocked_url = url(
                    "/webdriver/tests/support/http_handlers/authentication.py?"
                    f"username={username}&password={password}&realm={realm}"
                )
                if navigate:
                    # By default the authentication handler returns a text/plain
                    # content-type. Switch to text/html for a regular navigation.
                    blocked_url = f"{blocked_url}&contenttype=text/html"
            else:
                blocked_url = url("/webdriver/tests/bidi/network/support/empty.txt")

        await add_intercept(
            phases=[phase],
            url_patterns=[
                {
                    "type": "string",
                    "pattern": blocked_url,
                }
            ],
        )

        events = []

        async def on_event(method, data):
            events.append(data)

        remove_listener = bidi_session.add_event_listener(f"network.{phase}", on_event)

        network_event = wait_for_event(f"network.{phase}")
        if navigate:
            if navigate_url is None:
                navigate_url = blocked_url

            asyncio.ensure_future(
                bidi_session.browsing_context.navigate(
                    context=context["context"], url=navigate_url, wait="complete"
                )
            )
        else:
            asyncio.ensure_future(fetch(blocked_url, context=context, **kwargs))

        # Wait for the first blocked request. When testing a navigation where
        # navigate_url is different from blocked_url, non-blocked events will
        # be received before the blocked request.
        wait = AsyncPoll(bidi_session, timeout=2)
        await wait.until(lambda _: any(e["isBlocked"] is True for e in events))

        [blocked_event] = [e for e in events if e["isBlocked"] is True]
        request = blocked_event["request"]["request"]

        if phase == "authRequired":
            blocked_auth_requests.append(request)
        else:
            blocked_requests.append(request)

        return request

    yield setup_blocked_request

    # Cleanup unhandled blocked requests on teardown.
    for request in blocked_requests:
        try:
            await bidi_session.network.fail_request(request=request)
        except NoSuchRequestException:
            # Nothing to do here the request was probably handled during the test.
            pass

    # Cleanup unhandled blocked auth requests on teardown.
    for request in blocked_auth_requests:
        try:
            await bidi_session.network.continue_with_auth(
                request=request, action="cancel"
            )
        except NoSuchRequestException:
            # Nothing to do here the request was probably handled during the test.
            pass


@pytest.fixture
def origin(server_config, domain_value):
    def origin(protocol="https", domain="", subdomain=""):
        return urlunsplit((protocol, domain_value(domain, subdomain), "", "", ""))

    return origin
