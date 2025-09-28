import tempfile

import pytest
import pytest_asyncio

from webdriver import TimeoutException
from webdriver.bidi.modules.script import ContextTarget

DOWNLOAD_END = "browsingContext.downloadEnd"


@pytest.fixture
def temp_dir():
    return tempfile.mkdtemp()


@pytest_asyncio.fixture(params=["new", "default"])
async def user_context_invariant(request, create_user_context):
    if request.param == "default":
        return "default"
    return await create_user_context()


@pytest.fixture(params=[True, False])
def is_download_allowed_invariant(request, temp_dir):
    return request.param


@pytest.fixture
def some_download_behavior(is_download_allowed_invariant, temp_dir):
    """
    Returns a download behavior matching `is_download_allowed_invariant`.
    """
    if is_download_allowed_invariant:
        return {
            "type": "allowed",
            "destinationFolder": temp_dir
        }
    return {"type": "denied"}


@pytest.fixture
def opposite_download_behavior(is_download_allowed_invariant, temp_dir):
    """
    Returns a download behavior opposite to `is_download_allowed_invariant`.
    """
    if is_download_allowed_invariant:
        return {"type": "denied"}
    return {
        "type": "allowed",
        "destinationFolder": temp_dir
    }


@pytest.fixture
def trigger_download(bidi_session, subscribe_events, wait_for_event,
        wait_for_future_safe, inline):
    """
    Triggers download and returns either `browsingContext.downloadEnd` event or
    None if the download was not ended (e.g. if User Agent showed file save
    dialog).
    """

    async def trigger_download(context):
        page_with_download_link = inline(
            f"""<a id="download_link" href="{inline("")}" download="some_file.txt">download</a>""")
        await bidi_session.browsing_context.navigate(context=context["context"],
                                                     url=page_with_download_link,
                                                     wait="complete")

        await subscribe_events(events=[DOWNLOAD_END])

        on_download_will_begin = wait_for_event(DOWNLOAD_END)
        # Trigger download by clicking the link.
        await bidi_session.script.evaluate(
            expression="download_link.click()",
            target=ContextTarget(context["context"]),
            await_promise=True,
            user_activation=True,
        )

        try:
            return await wait_for_future_safe(
                on_download_will_begin, timeout=0.5)
        except TimeoutException:
            # User Agent showed file save dialog.
            return None

    return trigger_download


@pytest.fixture
def is_download_allowed(trigger_download):
    """
    Returns True, if download is allowed, False if download is not allowed, or
    "timeout" if download is not finished (e.g. if User Agent showed file save
    dialog).
    """

    async def is_download_allowed(context):
        event = await trigger_download(context)
        if event is None:
            return "timeout"
        return event["status"] == "complete"

    return is_download_allowed


@pytest_asyncio.fixture
async def default_is_download_allowed(is_download_allowed, new_tab):
    return await is_download_allowed(new_tab)
