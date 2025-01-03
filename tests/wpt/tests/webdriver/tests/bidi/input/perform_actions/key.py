import pytest

from webdriver.bidi.error import NoSuchFrameException
from webdriver.bidi.modules.input import Actions
from webdriver.bidi.modules.script import ContextTarget

from tests.support.keys import Keys
from .. import get_keys_value
from . import get_shadow_root_from_test_page

pytestmark = pytest.mark.asyncio

CONTEXT_LOAD_EVENT = "browsingContext.load"


async def test_invalid_browsing_context(bidi_session):
    actions = Actions()
    actions.add_key()

    with pytest.raises(NoSuchFrameException):
        await bidi_session.input.perform_actions(actions=actions, context="foo")


async def test_key_down_closes_browsing_context(
    bidi_session, configuration, new_tab, inline, subscribe_events,
    wait_for_event
):
    url = inline("""
        <input onkeydown="window.close()">close</input>
        <script>document.querySelector("input").focus();</script>
        """)

    # Opening a new context via `window.open` is required for script to be able
    # to close it.
    await subscribe_events(events=[CONTEXT_LOAD_EVENT])
    on_load = wait_for_event(CONTEXT_LOAD_EVENT)

    await bidi_session.script.evaluate(
        expression=f"window.open('{url}')",
        target=ContextTarget(new_tab["context"]),
        await_promise=True
    )
    # Wait for the new context to be created and get it.
    new_context = await on_load

    actions = Actions()
    (
        actions.add_key()
        .key_down("w")
        .pause(250 * configuration["timeout_multiplier"])
        .key_up("w")
    )

    with pytest.raises(NoSuchFrameException):
        await bidi_session.input.perform_actions(
            actions=actions, context=new_context["context"]
        )


async def test_key_backspace(bidi_session, top_context, setup_key_test):
    actions = Actions()
    actions.add_key().send_keys("efcd").send_keys([Keys.BACKSPACE, Keys.BACKSPACE])
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert keys_value == "ef"


@pytest.mark.parametrize(
    "value",
    [
        ("\U0001F604"),
        ("\U0001F60D"),
        ("\u0BA8\u0BBF"),
        ("\u1100\u1161\u11A8"),
    ],
)
async def test_key_codepoint(
    bidi_session, top_context, setup_key_test, value
):
    # Not using send_keys() because we always want to treat value as
    # one character here. `len(value)` varies by platform for non-BMP characters,
    # so we don't want to iterate over value.

    actions = Actions()
    (actions.add_key().key_down(value).key_up(value))
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    # events sent by major browsers are inconsistent so only check key value
    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert keys_value == value


@pytest.mark.parametrize("mode", ["open", "closed"])
@pytest.mark.parametrize("nested", [False, True], ids=["outer", "inner"])
async def test_key_shadow_tree(bidi_session, top_context, get_test_page, mode, nested):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=get_test_page(
            shadow_doc="<div><input type=text></div>",
            shadow_root_mode=mode,
            nested_shadow_dom=nested,
        ),
        wait="complete",
    )

    shadow_root = await get_shadow_root_from_test_page(bidi_session, top_context, nested)
    input_el = await bidi_session.script.call_function(
        function_declaration="""shadowRoot => {{
            const input = shadowRoot.querySelector('input');
            input.focus();
            return input;
        }}
        """,
        arguments=[shadow_root],
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    actions = Actions()
    (actions.add_key().key_down("a").key_up("a"))
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    input_value = await bidi_session.script.call_function(
        function_declaration="input => input.value",
        arguments=[input_el],
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert input_value["value"] == "a"


async def test_null_response_value(bidi_session, top_context):
    actions = Actions()
    actions.add_key().key_down("a").key_up("a")
    value = await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    assert value == {}
