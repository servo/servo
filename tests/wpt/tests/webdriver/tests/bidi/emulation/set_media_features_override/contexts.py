import pytest

from .conftest import (
    ANOTHER_FEATURE_NAME,
    ANOTHER_FEATURE_SOME_VALUE,
    SOME_FEATURE_NAME,
    SOME_FEATURE_SOME_VALUE,
)

pytestmark = pytest.mark.asyncio


async def test_contexts(
    bidi_session,
    new_tab,
    top_context,
    is_feature_enabled,
):
    default_is_some_feature_enabled = await is_feature_enabled(
        top_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
    )

    # Apply media features override to new_tab only.
    await bidi_session.emulation.set_media_features_override(
        contexts=[new_tab["context"]],
        features={SOME_FEATURE_NAME: SOME_FEATURE_SOME_VALUE},
    )
    assert (
        await is_feature_enabled(
            new_tab, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        is True
    )
    assert (
        await is_feature_enabled(
            top_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
    )

    # Verify newly created context remains in default state.
    another_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )
    assert (
        await is_feature_enabled(
            another_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
    )

    # Clear context-scoped override and verify all contexts revert to default.
    await bidi_session.emulation.set_media_features_override(
        contexts=[new_tab["context"]],
        features=None,
    )
    assert (
        await is_feature_enabled(
            new_tab, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
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


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_iframe(
    bidi_session,
    new_tab,
    top_context,
    url,
    create_iframe,
    domain,
    is_feature_enabled,
):
    default_is_some_feature_enabled = await is_feature_enabled(
        top_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
    )

    # Apply media features override to parent context.
    await bidi_session.emulation.set_media_features_override(
        contexts=[new_tab["context"]],
        features={SOME_FEATURE_NAME: SOME_FEATURE_SOME_VALUE},
    )
    assert (
        await is_feature_enabled(
            new_tab, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        is True
    )

    # Verify child iframe inherits media features override from top-level context.
    iframe = await create_iframe(new_tab, url("/", domain=domain))
    assert (
        await is_feature_enabled(
            iframe, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        is True
    )

    # Clear parent context override and verify both parent and iframe revert to default.
    await bidi_session.emulation.set_media_features_override(
        contexts=[new_tab["context"]],
        features=None,
    )
    assert (
        await is_feature_enabled(
            new_tab, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
    )
    assert (
        await is_feature_enabled(
            iframe, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
    )


async def test_overrides_user_contexts(
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

    # Set override at both browsing context and user context levels.
    await bidi_session.emulation.set_media_features_override(
        contexts=[context_in_user_context["context"]],
        features={SOME_FEATURE_NAME: SOME_FEATURE_SOME_VALUE},
    )
    await bidi_session.emulation.set_media_features_override(
        user_contexts=[affected_user_context],
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

    # Clear context-level setting and verify fallback to user-context setting.
    await bidi_session.emulation.set_media_features_override(
        contexts=[context_in_user_context["context"]],
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

    # Clear user-context setting and verify fallback to default state.
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
        == default_is_another_feature_enabled
    )


async def test_overrides_global(
    bidi_session,
    new_tab,
    top_context,
    is_feature_enabled,
):
    default_is_some_feature_enabled = await is_feature_enabled(
        top_context, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
    )
    default_is_another_feature_enabled = await is_feature_enabled(
        top_context, ANOTHER_FEATURE_NAME, ANOTHER_FEATURE_SOME_VALUE
    )

    # Set override at both browsing context and global levels.
    await bidi_session.emulation.set_media_features_override(
        contexts=[new_tab["context"]],
        features={SOME_FEATURE_NAME: SOME_FEATURE_SOME_VALUE},
    )
    await bidi_session.emulation.set_media_features_override(
        features={ANOTHER_FEATURE_NAME: ANOTHER_FEATURE_SOME_VALUE},
    )
    assert (
        await is_feature_enabled(
            new_tab, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        is True
    )
    assert (
        await is_feature_enabled(
            new_tab, ANOTHER_FEATURE_NAME, ANOTHER_FEATURE_SOME_VALUE
        )
        == default_is_another_feature_enabled
    )

    # Clear context-level setting and verify fallback to active global setting.
    await bidi_session.emulation.set_media_features_override(
        contexts=[new_tab["context"]],
        features=None,
    )
    assert (
        await is_feature_enabled(
            new_tab, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
    )
    assert (
        await is_feature_enabled(
            new_tab, ANOTHER_FEATURE_NAME, ANOTHER_FEATURE_SOME_VALUE
        )
        is True
    )

    # Clear global setting and verify fallback to default state.
    await bidi_session.emulation.set_media_features_override(
        features=None,
    )
    assert (
        await is_feature_enabled(
            new_tab, SOME_FEATURE_NAME, SOME_FEATURE_SOME_VALUE
        )
        == default_is_some_feature_enabled
    )
    assert (
        await is_feature_enabled(
            new_tab, ANOTHER_FEATURE_NAME, ANOTHER_FEATURE_SOME_VALUE
        )
        == default_is_another_feature_enabled
    )
