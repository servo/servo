import pytest
from webdriver.error import NoSuchWindowException

from tests.support.sync import AsyncPoll

pytestmark = pytest.mark.asyncio


async def test_classic_switch_to_parent_no_browsing_context(bidi_session, current_session, url):
    # With WebDriver classic it cannot be checked if the parent frame is already
    # gone before switching to it. To prevent race conditions such a check needs
    # to be done via WebDriver BiDi.
    current_session.url = url("/webdriver/tests/support/html/frames.html")

    subframe = current_session.find.css("#sub-frame", all=False)
    current_session.switch_frame(subframe)

    deleteframe = current_session.find.css("#delete-frame", all=False)
    current_session.switch_frame(deleteframe)

    button = current_session.find.css("#remove-top", all=False)
    button.click()

    async def is_frame_removed(_):
        contexts = await bidi_session.browsing_context.get_tree(root=current_session.window_handle)
        return not contexts[0]["children"]

    # Wait until IFrame is gone.
    wait = AsyncPoll(
        current_session,
        timeout=5,
        message="IFrame that should be closed is still open",
    )
    await wait.until(is_frame_removed)

    with pytest.raises(NoSuchWindowException):
        current_session.switch_frame("parent")
