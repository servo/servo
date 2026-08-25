import pytest

from webdriver.bidi.modules.input import Actions

from tests.support.keys import Keys
from .. import get_keys_value

pytestmark = pytest.mark.asyncio


async def test_meta_or_ctrl_with_printable_and_backspace_deletes_all_text(
    bidi_session, top_context, setup_key_test, modifier_key
):
    actions = Actions()
    (
        actions.add_key()
        .send_keys("abc d")
        .key_down(modifier_key)
        .key_down("a")
        .key_up(modifier_key)
        .key_up("a")
        .key_down(Keys.BACKSPACE)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert keys_value == ""


async def test_meta_or_ctrl_with_printable_cut_and_paste_text(
    bidi_session, top_context, setup_key_test, modifier_key
):
    initial = "abc d"
    actions = Actions()
    (
        actions.add_key()
        .send_keys(initial)
        .key_down(modifier_key)
        .key_down("a")
        .key_up(modifier_key)
        .key_up("a")
        .key_down(modifier_key)
        .key_down("x")
        .key_up(modifier_key)
        .key_up("x")
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert keys_value == ""

    actions = Actions()
    (
        actions.add_key()
        .key_down(modifier_key)
        .key_down("v")
        .key_up(modifier_key)
        .key_up("v")
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert keys_value == initial


async def test_meta_or_ctrl_with_printable_copy_and_paste_text(
    bidi_session, top_context, setup_key_test, modifier_key
):
    initial = "abc d"
    actions = Actions()
    (
        actions.add_key()
        .send_keys(initial)
        .key_down(modifier_key)
        .key_down("a")
        .key_up(modifier_key)
        .key_up("a")
        .key_down(modifier_key)
        .key_down("c")
        .key_up(modifier_key)
        .key_up("c")
        .send_keys([Keys.RIGHT])
        .key_down(modifier_key)
        .key_down("v")
        .key_up(modifier_key)
        .key_up("v")
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert keys_value == initial * 2


@pytest.mark.parametrize("modifier", [Keys.SHIFT, Keys.R_SHIFT])
async def test_key_modifier_shift_non_printable_keys(
    bidi_session, top_context, setup_key_test, modifier
):
    actions = Actions()
    (
        actions.add_key()
        .key_down("f")
        .key_up("f")
        .key_down("o")
        .key_up("o")
        .key_down("o")
        .key_up("o")
        .key_down(modifier)
        .key_down(Keys.BACKSPACE)
        .key_up(modifier)
        .key_up(Keys.BACKSPACE)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    keys_value = await get_keys_value(bidi_session, top_context["context"])

    assert keys_value == "fo"


@pytest.mark.parametrize("modifier", [Keys.SHIFT, Keys.R_SHIFT])
async def test_key_modifier_shift_printable_keys(
    bidi_session, top_context, setup_key_test, modifier
):
    actions = Actions()
    (
        actions.add_key()
        .key_down("b")
        .key_up("b")
        .key_down(modifier)
        .key_down("c")
        .key_up(modifier)
        .key_up("c")
        .key_down("d")
        .key_up("d")
        .key_down(modifier)
        .key_down("e")
        .key_up("e")
        .key_down("f")
        .key_up(modifier)
        .key_up("f")
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    keys_value = await get_keys_value(bidi_session, top_context["context"])

    assert keys_value == "bCdEF"
