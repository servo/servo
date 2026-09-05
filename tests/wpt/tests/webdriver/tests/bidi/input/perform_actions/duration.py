import pytest
import pytest_asyncio

from webdriver.bidi.modules.input import Actions
from webdriver.bidi.modules.script import ContextTarget

from .. import wait_for_events

pytestmark = pytest.mark.asyncio

# A move or scroll action with a duration greater than zero has to be split into
# multiple incremental events.
DURATION = 200


@pytest_asyncio.fixture
async def setup_event_recorder(bidi_session, top_context, inline):
    async def _setup_event_recorder(event_type):
        # "touch-action: none" keeps a synthesized touch drag from being
        # consumed as a scroll gesture, so the incremental touch events are
        # delivered to the content as "pointermove" events instead of a
        # "pointercancel".
        await bidi_session.browsing_context.navigate(
            context=top_context["context"],
            url=inline(
                "<style>:root { touch-action: none }</style>"
                "<body style='width: 1000px; height: 500px'></body>"
            ),
            wait="complete",
        )

        await bidi_session.script.call_function(
            function_declaration="""(eventType) => {
                window.allEvents = { events: [] };
                window.addEventListener(
                    eventType,
                    event => window.allEvents.events.push({
                        type: event.type,
                        clientX: event.clientX,
                        clientY: event.clientY,
                        deltaX: event.deltaX,
                        deltaY: event.deltaY,
                    })
                );
            }""",
            arguments=[{"type": "string", "value": event_type}],
            await_promise=False,
            target=ContextTarget(top_context["context"]),
        )

    return _setup_event_recorder


async def test_mouse_move_dispatches_multiple_events(
    bidi_session, top_context, setup_event_recorder
):
    await setup_event_recorder("mousemove")

    actions = Actions()
    actions.add_pointer().pointer_move(x=100, y=200, duration=DURATION)
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await wait_for_events(bidi_session, top_context["context"], min_count=2)
    assert len(events) > 1
    assert all(event["type"] == "mousemove" for event in events)

    # The last event has to reach the requested target position.
    assert events[-1]["clientX"] == pytest.approx(100, abs=1.0)
    assert events[-1]["clientY"] == pytest.approx(200, abs=1.0)


async def test_touch_move_dispatches_multiple_events(
    bidi_session, top_context, setup_event_recorder
):
    await setup_event_recorder("pointermove")

    actions = Actions()
    (
        actions.add_pointer(pointer_type="touch")
        .pointer_down(button=0)
        .pointer_move(x=200, y=100, duration=DURATION)
        .pointer_up(button=0)
    )
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await wait_for_events(bidi_session, top_context["context"], min_count=2)
    assert len(events) > 1
    assert all(event["type"] == "pointermove" for event in events)

    # The last event has to reach the requested target position.
    assert events[-1]["clientX"] == pytest.approx(200, abs=1.0)
    assert events[-1]["clientY"] == pytest.approx(100, abs=1.0)


async def test_wheel_scroll_dispatches_multiple_events(
    bidi_session, top_context, setup_event_recorder
):
    await setup_event_recorder("wheel")

    actions = Actions()
    actions.add_wheel().scroll(x=0, y=0, delta_x=30, delta_y=60, duration=DURATION)
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await wait_for_events(bidi_session, top_context["context"], min_count=2)
    assert len(events) > 1
    assert all(event["type"] == "wheel" for event in events)

    # The incremental deltas of all dispatched events have to add up to the
    # requested scroll distance.
    assert sum(event["deltaX"] for event in events) == 30
    assert sum(event["deltaY"] for event in events) == 60
