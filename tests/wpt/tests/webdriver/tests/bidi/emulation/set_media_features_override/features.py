import pytest

pytestmark = pytest.mark.asyncio

MEDIA_FEATURE_VALUES = [
    ("any-hover", "none"),
    ("any-pointer", "coarse"),
    ("color", 0),
    ("color-gamut", "rec2020"),
    ("color-index", 256),
    ("display-mode", "standalone"),
    ("dynamic-range", "high"),
    ("environment-blending", "additive"),
    ("forced-colors", "active"),
    ("grid", 1),
    ("horizontal-viewport-segments", 2),
    ("hover", "none"),
    ("inverted-colors", "inverted"),
    ("monochrome", 8),
    ("nav-controls", "none"),
    ("overflow-block", "paged"),
    ("overflow-inline", "none"),
    ("pointer", "coarse"),
    ("prefers-color-scheme", "dark"),
    ("prefers-contrast", "more"),
    ("prefers-reduced-data", "reduce"),
    ("prefers-reduced-motion", "reduce"),
    ("prefers-reduced-transparency", "reduce"),
    ("scan", "interlace"),
    ("scripting", "none"),
    ("update", "slow"),
    ("vertical-viewport-segments", 2),
    ("video-color-gamut", "rec2020"),
    ("video-dynamic-range", "high"),
]


@pytest.mark.parametrize(
    "feature, value",
    MEDIA_FEATURE_VALUES,
    ids=[f"{feature}={value}" for feature, value in MEDIA_FEATURE_VALUES],
)
async def test_features_values(
    bidi_session,
    new_tab,
    top_context,
    is_feature_enabled,
    feature,
    value,
):
    default_state = await is_feature_enabled(top_context, feature, value)

    # Apply media feature override and verify query matches.
    await bidi_session.emulation.set_media_features_override(
        contexts=[new_tab["context"]],
        features={feature: value},
    )
    assert await is_feature_enabled(new_tab, feature, value) is True

    # Reset individual feature using null value and verify reversion to default.
    await bidi_session.emulation.set_media_features_override(
        contexts=[new_tab["context"]],
        features={feature: None},
    )
    assert await is_feature_enabled(new_tab, feature, value) == default_state

    # Clear media features override and verify default state.
    await bidi_session.emulation.set_media_features_override(
        contexts=[new_tab["context"]],
        features=None,
    )
    assert await is_feature_enabled(new_tab, feature, value) == default_state
