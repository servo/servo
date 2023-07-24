import base64

from tests.support.asserts import assert_pdf
from tests.support.image import cm_to_px, png_dimensions, ImageDifference
from typing import Any, Mapping

import pytest
import pytest_asyncio
from webdriver.bidi.error import (
    InvalidArgumentException,
    NoSuchFrameException,
    NoSuchScriptException,
)
from webdriver.bidi.modules.script import ContextTarget


@pytest_asyncio.fixture
async def add_preload_script(bidi_session):
    preload_scripts_ids = []

    async def add_preload_script(function_declaration, arguments=None, sandbox=None):
        script = await bidi_session.script.add_preload_script(
            function_declaration=function_declaration,
            arguments=arguments,
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
def current_time(bidi_session, top_context):
    """Get the current time stamp in ms from the remote end.

    This is required especially when tests are run on different devices like
    for Android, where it's not guaranteed that both machines are in sync.
    """
    async def _():
        result = await bidi_session.script.evaluate(
            expression="Date.now()",
            target=ContextTarget(top_context["context"]),
            await_promise=True)
        return result["value"]

    return _


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
            target={"context": context["context"]},
            await_promise=True)
        iframe_dom_id = resp["value"]

        new_contexts = await bidi_session.browsing_context.get_tree(root=context["context"])
        added_contexts = ({item["context"] for item in new_contexts[0]["children"]} -
                          {item["context"] for item in initial_contexts[0]["children"]})
        assert len(added_contexts) == 1
        frame_id = added_contexts.pop()

        await bidi_session.script.evaluate(
            expression=f"document.getElementById('{iframe_dom_id}').remove()",
            target={"context": context["context"]},
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
