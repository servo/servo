import pytest

from webdriver.bidi.modules.script import ContextTarget
from webdriver.bidi.undefined import UNDEFINED
from tests.bidi.input.file_dialog_opened import assert_file_dialog_opened_event

pytestmark = pytest.mark.asyncio

FILE_DIALOG_OPENED_EVENT = "input.fileDialogOpened"


@pytest.mark.parametrize("multiple", [True, False])
async def test_show_open_file_picker(bidi_session, subscribe_events, inline,
        top_context, wait_for_event, wait_for_future_safe, multiple):
    await subscribe_events(events=[FILE_DIALOG_OPENED_EVENT])
    on_file_dialog_opened = wait_for_event(FILE_DIALOG_OPENED_EVENT)

    # Navigate to a page to enable file picker.
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=(inline("")),
                                                 wait="complete")

    await bidi_session.script.evaluate(
        expression=f"window.showOpenFilePicker({{'multiple': {'true' if multiple else 'false'}}})",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        user_activation=True
    )

    event = await wait_for_future_safe(on_file_dialog_opened)
    assert_file_dialog_opened_event(event, top_context["context"],
                                    multiple=multiple, element=UNDEFINED)
