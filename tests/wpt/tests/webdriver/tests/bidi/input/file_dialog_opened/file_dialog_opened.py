import pytest
from webdriver.bidi.modules.script import ContextTarget
from webdriver.error import TimeoutException

from tests.bidi import wait_for_bidi_events
from . import assert_file_dialog_opened_event


pytestmark = pytest.mark.asyncio

FILE_DIALOG_OPENED_EVENT = "input.fileDialogOpened"


async def test_unsubscribe(bidi_session, inline, top_context, wait_for_event,
        wait_for_future_safe):
    await bidi_session.session.subscribe(events=[FILE_DIALOG_OPENED_EVENT])
    await bidi_session.session.unsubscribe(events=[FILE_DIALOG_OPENED_EVENT])

    # Track all received input.fileDialogOpened events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(FILE_DIALOG_OPENED_EVENT,
                                                      on_event)

    url = inline("<input id=input type=file />")
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=url, wait="complete")

    await bidi_session.script.evaluate(
        expression="input.click()",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        user_activation=True
    )

    with pytest.raises(TimeoutException):
        await wait_for_bidi_events(bidi_session, events, 1, timeout=0.5)

    remove_listener()


async def test_subscribe(bidi_session, subscribe_events, inline, top_context,
        wait_for_event, wait_for_future_safe):
    await subscribe_events(events=[FILE_DIALOG_OPENED_EVENT])
    on_entry = wait_for_event(FILE_DIALOG_OPENED_EVENT)

    url = inline("<input id=input type=file />")
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=url, wait="complete")

    await bidi_session.script.evaluate(
        expression="input.click()",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        user_activation=True
    )

    event = await wait_for_future_safe(on_entry)
    assert_file_dialog_opened_event(event, top_context["context"])


async def test_show_picker(bidi_session, subscribe_events, inline, top_context,
        wait_for_event, wait_for_future_safe):
    await subscribe_events(events=[FILE_DIALOG_OPENED_EVENT])
    on_entry = wait_for_event(FILE_DIALOG_OPENED_EVENT)

    url = inline("<input id=input type=file />")
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=url, wait="complete")

    await bidi_session.script.evaluate(
        expression="input.showPicker()",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        user_activation=True
    )

    event = await wait_for_future_safe(on_entry)
    assert_file_dialog_opened_event(event, top_context["context"])


@pytest.mark.parametrize("multiple", [True, False])
async def test_multiple(bidi_session, subscribe_events, inline, top_context,
        wait_for_event, wait_for_future_safe, multiple):
    await subscribe_events(events=[FILE_DIALOG_OPENED_EVENT])
    on_entry = wait_for_event(FILE_DIALOG_OPENED_EVENT)

    url = inline(
        f"<input id=input type=file {'multiple' if multiple else ''} />")
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=url, wait="complete")

    await bidi_session.script.evaluate(
        expression="input.click()",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        user_activation=True
    )
    event = await wait_for_future_safe(on_entry)
    assert_file_dialog_opened_event(event, top_context["context"],
                                    multiple=multiple)


async def test_element(bidi_session, subscribe_events, inline, top_context,
        wait_for_event, wait_for_future_safe):
    await subscribe_events(events=[FILE_DIALOG_OPENED_EVENT])
    on_entry = wait_for_event(FILE_DIALOG_OPENED_EVENT)

    url = inline("<input id=input type=file />")
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=url, wait="complete")

    node = await bidi_session.script.evaluate(
        expression="input.click(); input",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        user_activation=True
    )

    event = await wait_for_future_safe(on_entry)
    expected_element = {
        'sharedId': node["sharedId"],
    }
    assert_file_dialog_opened_event(event, top_context["context"],
                                    element=expected_element)
