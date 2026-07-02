import pytest
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio

FILE_DIALOG_OPENED_EVENT = "input.fileDialogOpened"


@pytest.mark.capabilities(
    {"unhandledPromptBehavior": {'file': 'dismiss', 'default': 'ignore'}})
@pytest.mark.parametrize("user_context", ["default", "new"])
async def test_user_context(
    bidi_session,
    subscribe_events,
    create_user_context,
    inline,
    wait_for_event,
    wait_for_future_safe,
    user_context,
):
    user_context_id = (
        await create_user_context() if user_context == "new" else user_context
    )

    context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context_id
    )

    await bidi_session.browsing_context.navigate(
        context=context["context"],
        url=inline("<input type='file'>"),
        wait="complete",
    )
    await subscribe_events(events=[FILE_DIALOG_OPENED_EVENT])
    on_entry = wait_for_event(FILE_DIALOG_OPENED_EVENT)
    await bidi_session.script.evaluate(
        expression="document.querySelector('input[type=file]').click()",
        target=ContextTarget(context["context"]),
        await_promise=False,
        user_activation=True,
    )
    event = await wait_for_future_safe(on_entry)
    assert "userContext" in event
    assert event["userContext"] == user_context_id
