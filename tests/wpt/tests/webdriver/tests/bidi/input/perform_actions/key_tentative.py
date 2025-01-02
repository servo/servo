import pytest

from webdriver.bidi.error import NoSuchFrameException
from webdriver.bidi.modules.input import Actions
from webdriver.bidi.modules.script import ContextTarget

from tests.support.keys import Keys
from .. import get_keys_value
from . import get_shadow_root_from_test_page

pytestmark = pytest.mark.asyncio

@pytest.mark.parametrize(
    "value",
    [
        ("\u0e01\u0e33"),
        ("🤷🏽‍♀️"),
    ],
)
async def test_grapheme_cluster(
    bidi_session, top_context, setup_key_test, value
):
    actions = Actions()
    (actions.add_key().key_down(value).key_up(value))
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    # events sent by major browsers are inconsistent so only check key value
    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert keys_value == value
