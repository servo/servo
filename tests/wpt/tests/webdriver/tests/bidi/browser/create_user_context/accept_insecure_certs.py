import pytest

from .. import get_user_context_ids

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("accept_insecure_certs", [True, False])
async def test_create_context(bidi_session, create_user_context,
        accept_insecure_certs):
    user_context = await create_user_context(
        accept_insecure_certs=accept_insecure_certs)
    # TODO: check the parameter is respected.
    assert user_context in await get_user_context_ids(bidi_session)
