import pytest

from .conftest import SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE

pytestmark = pytest.mark.asyncio


async def test_global(
    bidi_session,
    top_context,
    is_feature_enabled,
):
    default_is_some_feature_enabled = await is_feature_enabled(
        top_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
    )

    # Apply global media features override.
    await bidi_session.emulation.set_media_features_override(
        features={SOME_FEATURE_NAME: SOME_FEATURE_SOME_VALUE},
    )
    assert (
        await is_feature_enabled(
            top_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        is True
    )

    # Verify newly created context inherits global override.
    another_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )
    assert (
        await is_feature_enabled(
            another_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        is True
    )

    # Clear global setting and verify all contexts revert to default state.
    await bidi_session.emulation.set_media_features_override(
        features=None,
    )
    assert (
        await is_feature_enabled(
            top_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
    )
    assert (
        await is_feature_enabled(
            another_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
    )
