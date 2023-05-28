import pytest

from webdriver.bidi.modules.input import Actions

from tests.support.keys import Keys
from .. import get_keys_value

pytestmark = pytest.mark.asyncio


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


async def test_null_response_value(bidi_session, top_context):
    actions = Actions()
    actions.add_key().key_down("a").key_up("a")
    value = await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    assert value == {}
