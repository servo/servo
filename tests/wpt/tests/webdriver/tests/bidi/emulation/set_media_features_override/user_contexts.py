import pytest

from .conftest import (
    ANOTHER_FEATURE_NAME,
    ANOTHER_FEATURE_SOME_VALUE,
    SOME_FEATURE_NAME,
    SOME_FEATURE_SOME_VALUE,
)

pytestmark = pytest.mark.asyncio


async def test_user_contexts(
    bidi_session,
    top_context,
    affected_user_context,
    not_affected_user_context,
    is_feature_enabled,
):
    default_is_some_feature_enabled = await is_feature_enabled(
        top_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
    )

    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=affected_user_context,
        type_hint="tab",
    )
    context_in_other_user_context = await bidi_session.browsing_context.create(
        user_context=not_affected_user_context,
        type_hint="tab",
    )

    # Apply media features override to affected_user_context.
    await bidi_session.emulation.set_media_features_override(
        user_contexts=[affected_user_context],
        features={SOME_FEATURE_NAME: SOME_FEATURE_SOME_VALUE},
    )
    assert (
        await is_feature_enabled(
            context_in_user_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        is True
    )
    assert (
        await is_feature_enabled(
            context_in_other_user_context,
            SOME_FEATURE_NAME,
            SOME_FEATURE_SOME_VALUE,
        )
        == default_is_some_feature_enabled
    )

    # Verify newly created context in affected_user_context inherits override.
    another_context_in_user_context = (
        await bidi_session.browsing_context.create(
            user_context=affected_user_context,
            type_hint="tab",
        )
    )
    assert (
        await is_feature_enabled(
            another_context_in_user_context,
            SOME_FEATURE_NAME,
            SOME_FEATURE_SOME_VALUE,
        )
        is True
    )

    # Verify newly created context in not_affected_user_context stays default.
    another_context_in_other_user_context = (
        await bidi_session.browsing_context.create(
            user_context=not_affected_user_context,
            type_hint="tab",
        )
    )
    assert (
        await is_feature_enabled(
            another_context_in_other_user_context,
            SOME_FEATURE_NAME,
            SOME_FEATURE_SOME_VALUE,
        )
        == default_is_some_feature_enabled
    )

    # Clear user context setting and verify all contexts revert to default.
    await bidi_session.emulation.set_media_features_override(
        user_contexts=[affected_user_context],
        features=None,
    )
    assert (
        await is_feature_enabled(
            context_in_user_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
    )
    assert (
        await is_feature_enabled(
            another_context_in_user_context,
            SOME_FEATURE_NAME,
            SOME_FEATURE_SOME_VALUE,
        )
        == default_is_some_feature_enabled
    )
    assert (
        await is_feature_enabled(
            context_in_other_user_context,
            SOME_FEATURE_NAME,
            SOME_FEATURE_SOME_VALUE,
        )
        == default_is_some_feature_enabled
    )
    assert (
        await is_feature_enabled(
            another_context_in_other_user_context,
            SOME_FEATURE_NAME,
            SOME_FEATURE_SOME_VALUE,
        )
        == default_is_some_feature_enabled
    )


async def test_overrides_global(
    bidi_session,
    top_context,
    affected_user_context,
    is_feature_enabled,
):
    default_is_some_feature_enabled = await is_feature_enabled(
        top_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
    )
    default_is_another_feature_enabled = await is_feature_enabled(
        top_context, ANOTHER_FEATURE_NAME, ANOTHER_FEATURE_SOME_VALUE
    )

    context_in_user_context = await bidi_session.browsing_context.create(
        user_context=affected_user_context,
        type_hint="tab",
    )

    # Set override at both user context and global levels.
    await bidi_session.emulation.set_media_features_override(
        user_contexts=[affected_user_context],
        features={SOME_FEATURE_NAME: SOME_FEATURE_SOME_VALUE},
    )
    await bidi_session.emulation.set_media_features_override(
        features={ANOTHER_FEATURE_NAME: ANOTHER_FEATURE_SOME_VALUE},
    )
    assert (
        await is_feature_enabled(
            context_in_user_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        is True
    )
    assert (
        await is_feature_enabled(
            context_in_user_context,
            ANOTHER_FEATURE_NAME,
            ANOTHER_FEATURE_SOME_VALUE,
        )
        == default_is_another_feature_enabled
    )

    # Clear user-context setting and verify fallback to active global setting.
    await bidi_session.emulation.set_media_features_override(
        user_contexts=[affected_user_context],
        features=None,
    )
    assert (
        await is_feature_enabled(
            context_in_user_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
    )
    assert (
        await is_feature_enabled(
            context_in_user_context,
            ANOTHER_FEATURE_NAME,
            ANOTHER_FEATURE_SOME_VALUE,
        )
        is True
    )

    # Clear global setting and verify fallback to default state.
    await bidi_session.emulation.set_media_features_override(
        features=None,
    )
    assert (
        await is_feature_enabled(
            context_in_user_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
    )
    assert (
        await is_feature_enabled(
            context_in_user_context,
            ANOTHER_FEATURE_NAME,
            ANOTHER_FEATURE_SOME_VALUE,
        )
        == default_is_another_feature_enabled
    )
