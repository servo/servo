/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%! from data import to_rust_ident %>
<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import ALL_SIZES, PHYSICAL_SIDES, LOGICAL_SIDES %>

<% data.new_style_struct("Position", inherited=False) %>

// "top" / "left" / "bottom" / "right"
% for side in PHYSICAL_SIDES:
    ${helpers.predefined_type(
        side,
        "LengthPercentageOrAuto",
        "computed::LengthPercentageOrAuto::auto()",
        engines="gecko servo-2013 servo-2020",
        spec="https://www.w3.org/TR/CSS2/visuren.html#propdef-%s" % side,
        animation_value_type="ComputedValue",
        allow_quirks="Yes",
        servo_restyle_damage="reflow_out_of_flow",
        logical_group="inset",
    )}
% endfor
// inset-* logical properties, map to "top" / "left" / "bottom" / "right"
% for side in LOGICAL_SIDES:
    ${helpers.predefined_type(
        "inset-%s" % side,
        "LengthPercentageOrAuto",
        "computed::LengthPercentageOrAuto::auto()",
        engines="gecko servo-2013 servo-2020",
        spec="https://drafts.csswg.org/css-logical-props/#propdef-inset-%s" % side,
        alias="offset-%s:layout.css.offset-logical-properties.enabled" % side,
        animation_value_type="ComputedValue",
        logical=True,
        logical_group="inset",
    )}
% endfor

#[cfg(feature = "gecko")]
macro_rules! impl_align_conversions {
    ($name: path) => {
        impl From<u8> for $name {
            fn from(bits: u8) -> $name {
                $name(crate::values::specified::align::AlignFlags::from_bits(bits)
                      .expect("bits contain valid flag"))
            }
        }

        impl From<$name> for u8 {
            fn from(v: $name) -> u8 {
                v.0.bits()
            }
        }
    };
}

${helpers.predefined_type(
    "z-index",
    "ZIndex",
    "computed::ZIndex::auto()",
    engines="gecko servo-2013 servo-2020",
    spec="https://www.w3.org/TR/CSS2/visuren.html#z-index",
    flags="CREATES_STACKING_CONTEXT",
    animation_value_type="ComputedValue",
)}

// CSS Flexible Box Layout Module Level 1
// http://www.w3.org/TR/css3-flexbox/

// Flex container properties
${helpers.single_keyword(
    "flex-direction",
    "row row-reverse column column-reverse",
    engines="gecko servo-2013",
    spec="https://drafts.csswg.org/css-flexbox/#flex-direction-property",
    extra_prefixes="webkit",
    animation_value_type="discrete",
    servo_restyle_damage = "reflow",
    gecko_enum_prefix = "StyleFlexDirection",
)}

${helpers.single_keyword(
    "flex-wrap",
    "nowrap wrap wrap-reverse",
    engines="gecko servo-2013",
    spec="https://drafts.csswg.org/css-flexbox/#flex-wrap-property",
    extra_prefixes="webkit",
    animation_value_type="discrete",
    servo_restyle_damage = "reflow",
    gecko_enum_prefix = "StyleFlexWrap",
)}

% if engine == "servo-2013":
    // FIXME: Update Servo to support the same Syntax as Gecko.
    ${helpers.single_keyword(
        "justify-content",
        "flex-start stretch flex-end center space-between space-around",
        engines="servo-2013",
        extra_prefixes="webkit",
        spec="https://drafts.csswg.org/css-align/#propdef-justify-content",
        animation_value_type="discrete",
        servo_restyle_damage = "reflow",
    )}
% endif
% if engine == "gecko":
    ${helpers.predefined_type(
        "justify-content",
        "JustifyContent",
        "specified::JustifyContent(specified::ContentDistribution::normal())",
        engines="gecko",
        spec="https://drafts.csswg.org/css-align/#propdef-justify-content",
        extra_prefixes="webkit",
        animation_value_type="discrete",
        servo_restyle_damage="reflow",
    )}

    ${helpers.predefined_type(
        "justify-tracks",
        "JustifyTracks",
        "specified::JustifyTracks::default()",
        engines="gecko",
        gecko_pref="layout.css.grid-template-masonry-value.enabled",
        animation_value_type="discrete",
        servo_restyle_damage="reflow",
        spec="https://github.com/w3c/csswg-drafts/issues/4650",
    )}
% endif

% if engine in ["servo-2013", "servo-2020"]:
    // FIXME: Update Servo to support the same Syntax as Gecko.
    ${helpers.single_keyword(
        "align-content",
        "stretch flex-start flex-end center space-between space-around",
        engines="servo-2013",
        extra_prefixes="webkit",
        spec="https://drafts.csswg.org/css-align/#propdef-align-content",
        animation_value_type="discrete",
        servo_restyle_damage="reflow",
    )}

    ${helpers.single_keyword(
        "align-items",
        "stretch flex-start flex-end center baseline",
        engines="servo-2013 servo-2020",
        servo_2020_pref="layout.2020.unimplemented",
        extra_prefixes="webkit",
        spec="https://drafts.csswg.org/css-flexbox/#align-items-property",
        animation_value_type="discrete",
        servo_restyle_damage="reflow",
    )}
% endif
% if engine == "gecko":
    ${helpers.predefined_type(
        "align-content",
        "AlignContent",
        "specified::AlignContent(specified::ContentDistribution::normal())",
        engines="gecko",
        spec="https://drafts.csswg.org/css-align/#propdef-align-content",
        extra_prefixes="webkit",
        animation_value_type="discrete",
        servo_restyle_damage="reflow",
    )}

    ${helpers.predefined_type(
        "align-tracks",
        "AlignTracks",
        "specified::AlignTracks::default()",
        engines="gecko",
        gecko_pref="layout.css.grid-template-masonry-value.enabled",
        animation_value_type="discrete",
        servo_restyle_damage="reflow",
        spec="https://github.com/w3c/csswg-drafts/issues/4650",
    )}

    ${helpers.predefined_type(
        "align-items",
        "AlignItems",
        "specified::AlignItems::normal()",
        engines="gecko",
        spec="https://drafts.csswg.org/css-align/#propdef-align-items",
        extra_prefixes="webkit",
        animation_value_type="discrete",
        servo_restyle_damage="reflow",
    )}

    #[cfg(feature = "gecko")]
    impl_align_conversions!(crate::values::specified::align::AlignItems);

    ${helpers.predefined_type(
        "justify-items",
        "JustifyItems",
        "computed::JustifyItems::legacy()",
        engines="gecko",
        spec="https://drafts.csswg.org/css-align/#propdef-justify-items",
        animation_value_type="discrete",
    )}

    #[cfg(feature = "gecko")]
    impl_align_conversions!(crate::values::specified::align::JustifyItems);
% endif

// Flex item properties
${helpers.predefined_type(
    "flex-grow",
    "NonNegativeNumber",
    "From::from(0.0)",
    engines="gecko servo-2013",
    spec="https://drafts.csswg.org/css-flexbox/#flex-grow-property",
    extra_prefixes="webkit",
    animation_value_type="NonNegativeNumber",
    servo_restyle_damage="reflow",
)}

${helpers.predefined_type(
    "flex-shrink",
    "NonNegativeNumber",
    "From::from(1.0)",
    engines="gecko servo-2013",
    spec="https://drafts.csswg.org/css-flexbox/#flex-shrink-property",
    extra_prefixes="webkit",
    animation_value_type="NonNegativeNumber",
    servo_restyle_damage = "reflow",
)}

// https://drafts.csswg.org/css-align/#align-self-property
% if engine in ["servo-2013", "servo-2020"]:
    // FIXME: Update Servo to support the same syntax as Gecko.
    ${helpers.single_keyword(
        "align-self",
        "auto stretch flex-start flex-end center baseline",
        engines="servo-2013 servo-2020",
        servo_2020_pref="layout.2020.unimplemented",
        extra_prefixes="webkit",
        spec="https://drafts.csswg.org/css-flexbox/#propdef-align-self",
        animation_value_type="discrete",
        servo_restyle_damage = "reflow",
    )}
% endif
% if engine == "gecko":
    ${helpers.predefined_type(
        "align-self",
        "AlignSelf",
        "specified::AlignSelf(specified::SelfAlignment::auto())",
        engines="gecko",
        spec="https://drafts.csswg.org/css-align/#align-self-property",
        extra_prefixes="webkit",
        animation_value_type="discrete",
    )}

    ${helpers.predefined_type(
        "justify-self",
        "JustifySelf",
        "specified::JustifySelf(specified::SelfAlignment::auto())",
        engines="gecko",
        spec="https://drafts.csswg.org/css-align/#justify-self-property",
        animation_value_type="discrete",
    )}

    #[cfg(feature = "gecko")]
    impl_align_conversions!(crate::values::specified::align::SelfAlignment);
% endif

// https://drafts.csswg.org/css-flexbox/#propdef-order
${helpers.predefined_type(
    "order",
    "Integer",
    "0",
    engines="gecko servo-2013",
    extra_prefixes="webkit",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-flexbox/#order-property",
    servo_restyle_damage="reflow",
)}

${helpers.predefined_type(
    "flex-basis",
    "FlexBasis",
    "computed::FlexBasis::auto()",
    engines="gecko servo-2013",
    spec="https://drafts.csswg.org/css-flexbox/#flex-basis-property",
    extra_prefixes="webkit",
    animation_value_type="FlexBasis",
    servo_restyle_damage="reflow",
    boxed=True,
)}

% for (size, logical) in ALL_SIZES:
    <%
        spec = "https://drafts.csswg.org/css-box/#propdef-%s"
        if logical:
            spec = "https://drafts.csswg.org/css-logical-props/#propdef-%s"
    %>
    // width, height, block-size, inline-size
    ${helpers.predefined_type(
        size,
        "Size",
        "computed::Size::auto()",
        engines="gecko servo-2013 servo-2020",
        logical=logical,
        logical_group="size",
        allow_quirks="No" if logical else "Yes",
        spec=spec % size,
        animation_value_type="Size",
        servo_restyle_damage="reflow",
    )}
    // min-width, min-height, min-block-size, min-inline-size
    ${helpers.predefined_type(
        "min-%s" % size,
        "Size",
        "computed::Size::auto()",
        engines="gecko servo-2013 servo-2020",
        logical=logical,
        logical_group="min-size",
        allow_quirks="No" if logical else "Yes",
        spec=spec % size,
        animation_value_type="Size",
        servo_restyle_damage="reflow",
    )}
    ${helpers.predefined_type(
        "max-%s" % size,
        "MaxSize",
        "computed::MaxSize::none()",
        engines="gecko servo-2013 servo-2020",
        logical=logical,
        logical_group="max-size",
        allow_quirks="No" if logical else "Yes",
        spec=spec % size,
        animation_value_type="MaxSize",
        servo_restyle_damage="reflow",
    )}
% endfor

${helpers.single_keyword(
    "box-sizing",
    "content-box border-box",
    engines="gecko servo-2013 servo-2020",
    extra_prefixes="moz:layout.css.prefixes.box-sizing webkit",
    spec="https://drafts.csswg.org/css-ui/#propdef-box-sizing",
    gecko_enum_prefix="StyleBoxSizing",
    custom_consts={ "content-box": "Content", "border-box": "Border" },
    animation_value_type="discrete",
    servo_restyle_damage = "reflow",
)}

${helpers.single_keyword(
    "object-fit",
    "fill contain cover none scale-down",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-images/#propdef-object-fit",
    gecko_enum_prefix = "StyleObjectFit",
)}

${helpers.predefined_type(
    "object-position",
    "Position",
    "computed::Position::center()",
    engines="gecko",
    boxed=True,
    spec="https://drafts.csswg.org/css-images-3/#the-object-position",
    animation_value_type="ComputedValue",
)}

% for kind in ["row", "column"]:
    % for range in ["start", "end"]:
        ${helpers.predefined_type(
            "grid-%s-%s" % (kind, range),
            "GridLine",
            "Default::default()",
            engines="gecko",
            animation_value_type="discrete",
            spec="https://drafts.csswg.org/css-grid/#propdef-grid-%s-%s" % (kind, range),
        )}
    % endfor

    ${helpers.predefined_type(
        "grid-auto-%ss" % kind,
        "ImplicitGridTracks",
        "Default::default()",
        engines="gecko",
        animation_value_type="discrete",
        spec="https://drafts.csswg.org/css-grid/#propdef-grid-auto-%ss" % kind,
    )}

    ${helpers.predefined_type(
        "grid-template-%ss" % kind,
        "GridTemplateComponent",
        "specified::GenericGridTemplateComponent::None",
        engines="gecko",
        spec="https://drafts.csswg.org/css-grid/#propdef-grid-template-%ss" % kind,
        animation_value_type="ComputedValue",
    )}

% endfor

${helpers.predefined_type(
    "masonry-auto-flow",
    "MasonryAutoFlow",
    "computed::MasonryAutoFlow::initial()",
    engines="gecko",
    gecko_pref="layout.css.grid-template-masonry-value.enabled",
    animation_value_type="discrete",
    spec="https://github.com/w3c/csswg-drafts/issues/4650",
)}

${helpers.predefined_type(
    "grid-auto-flow",
    "GridAutoFlow",
    "computed::GridAutoFlow::ROW",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-grid/#propdef-grid-auto-flow",
)}

${helpers.predefined_type(
    "grid-template-areas",
    "GridTemplateAreas",
    "computed::GridTemplateAreas::none()",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-grid/#propdef-grid-template-areas",
)}

${helpers.predefined_type(
    "column-gap",
    "length::NonNegativeLengthPercentageOrNormal",
    "computed::length::NonNegativeLengthPercentageOrNormal::normal()",
    engines="gecko servo-2013",
    alias="grid-column-gap" if engine == "gecko" else "",
    extra_prefixes="moz:layout.css.prefixes.columns",
    servo_2013_pref="layout.columns.enabled",
    spec="https://drafts.csswg.org/css-align-3/#propdef-column-gap",
    animation_value_type="NonNegativeLengthPercentageOrNormal",
    servo_restyle_damage="reflow",
)}

// no need for -moz- prefixed alias for this property
${helpers.predefined_type(
    "row-gap",
    "length::NonNegativeLengthPercentageOrNormal",
    "computed::length::NonNegativeLengthPercentageOrNormal::normal()",
    engines="gecko",
    alias="grid-row-gap",
    spec="https://drafts.csswg.org/css-align-3/#propdef-row-gap",
    animation_value_type="NonNegativeLengthPercentageOrNormal",
    servo_restyle_damage="reflow",
)}

// NOTE(emilio): Before exposing this property to content, we probably need to
// change syntax and such, and make it apply to more elements.
//
// For now, it's used only for mapped attributes.
${helpers.predefined_type(
    "aspect-ratio",
    "Number",
    "computed::Number::zero()",
    engines="gecko servo-2013",
    animation_value_type="ComputedValue",
    spec="Internal, for now",
    enabled_in="",
    servo_restyle_damage="reflow",
)}
