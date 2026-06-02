import pytest

from webdriver.bidi.error import NoSuchFrameException
from webdriver.bidi.modules.input import Actions


pytestmark = pytest.mark.asyncio


async def test_invalid_browsing_context(bidi_session):
    actions = Actions()
    actions.add_pointer()

    with pytest.raises(NoSuchFrameException):
        await bidi_session.input.perform_actions(actions=actions, context="foo")
