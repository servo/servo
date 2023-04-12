import pytest

from webdriver.bidi.modules.input import Actions

pytestmark = pytest.mark.asyncio


async def test_null_response_value(bidi_session, top_context):
    actions = Actions()
    actions.add_key().key_down("a").key_up("a")
    value = await bidi_session.input.perform_actions(actions=actions,
                                                     context=top_context["context"])
    assert value == {}

