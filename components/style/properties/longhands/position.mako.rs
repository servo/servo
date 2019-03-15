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
        "computed::LengthPercentageOrAuto::Auto",
        spec="https://www.w3.org/TR/CSS2/visuren.html#propdef-%s" % side,
        flags="GETCS_NEEDS_LAYOUT_FLUSH",
        animation_value_type="ComputedValue",
        allow_quirks=True,
        servo_restyle_damage="reflow_out_of_flow",
        logical_group="inset",
    )}
% endfor
// inset-* logical properties, map to "top" / "left" / "bottom" / "right"
% for side in LOGICAL_SIDES:
    ${helpers.predefined_type(
        "inset-%s" % side,
        "LengthPercentageOrAuto",
        "computed::LengthPercentageOrAuto::Auto",
        spec="https://drafts.csswg.org/css-logical-props/#propdef-inset-%s" % side,
        flags="GETCS_NEEDS_LAYOUT_FLUSH",
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
    spec="https://drafts.csswg.org/css-flexbox/#flex-direction-property",
    extra_prefixes="webkit",
    animation_value_type="discrete",
    servo_restyle_damage = "reflow",
)}

${helpers.single_keyword(
    "flex-wrap",
    "nowrap wrap wrap-reverse",
    spec="https://drafts.csswg.org/css-flexbox/#flex-wrap-property",
    extra_prefixes="webkit",
    animation_value_type="discrete",
    servo_restyle_damage = "reflow",
)}

% if product == "servo":
    // FIXME: Update Servo to support the same Syntax as Gecko.
    ${helpers.single_keyword(
        "justify-content",
        "flex-start stretch flex-end center space-between space-around",
        extra_prefixes="webkit",
        spec="https://drafts.csswg.org/css-align/#propdef-justify-content",
        animation_value_type="discrete",
        servo_restyle_damage = "reflow",
    )}
% else:
    ${helpers.predefined_type(
        "justify-content",
        "JustifyContent",
        "specified::JustifyContent(specified::ContentDistribution::normal())",
        spec="https://drafts.csswg.org/css-align/#propdef-justify-content",
        extra_prefixes="webkit",
        animation_value_type="discrete",
        servo_restyle_damage="reflow",
    )}
% endif

% if product == "servo":
    // FIXME: Update Servo to support the same Syntax as Gecko.
    ${helpers.single_keyword(
        "align-content",
        "stretch flex-start flex-end center space-between space-around",
        extra_prefixes="webkit",
        spec="https://drafts.csswg.org/css-align/#propdef-align-content",
        animation_value_type="discrete",
        servo_restyle_damage="reflow",
    )}

    ${helpers.single_keyword(
        "align-items",
        "stretch flex-start flex-end center baseline",
        extra_prefixes="webkit",
        spec="https://drafts.csswg.org/css-flexbox/#align-items-property",
        animation_value_type="discrete",
        servo_restyle_damage="reflow",
    )}
% else:
    ${helpers.predefined_type(
        "align-content",
        "AlignContent",
        "specified::AlignContent(specified::ContentDistribution::normal())",
        spec="https://drafts.csswg.org/css-align/#propdef-align-content",
        extra_prefixes="webkit",
        animation_value_type="discrete",
        servo_restyle_damage="reflow",
    )}

    ${helpers.predefined_type(
        "align-items",
        "AlignItems",
        "specified::AlignItems::normal()",
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
    spec="https://drafts.csswg.org/css-flexbox/#flex-grow-property",
    extra_prefixes="webkit",
    animation_value_type="NonNegativeNumber",
    servo_restyle_damage="reflow",
)}

${helpers.predefined_type(
    "flex-shrink",
    "NonNegativeNumber",
    "From::from(1.0)",
    spec="https://drafts.csswg.org/css-flexbox/#flex-shrink-property",
    extra_prefixes="webkit",
    animation_value_type="NonNegativeNumber",
    servo_restyle_damage = "reflow",
)}

// https://drafts.csswg.org/css-align/#align-self-property
% if product == "servo":
    // FIXME: Update Servo to support the same syntax as Gecko.
    ${helpers.single_keyword(
        "align-self",
        "auto stretch flex-start flex-end center baseline",
        extra_prefixes="webkit",
        spec="https://drafts.csswg.org/css-flexbox/#propdef-align-self",
        animation_value_type="discrete",
        servo_restyle_damage = "reflow",
    )}
% else:
    ${helpers.predefined_type(
        "align-self",
        "AlignSelf",
        "specified::AlignSelf(specified::SelfAlignment::auto())",
        spec="https://drafts.csswg.org/css-align/#align-self-property",
        extra_prefixes="webkit",
        animation_value_type="discrete",
    )}

    ${helpers.predefined_type(
        "justify-self",
        "JustifySelf",
        "specified::JustifySelf(specified::SelfAlignment::auto())",
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
    extra_prefixes="webkit",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-flexbox/#order-property",
    servo_restyle_damage = "reflow",
)}

${helpers.predefined_type(
    "flex-basis",
    "FlexBasis",
    "computed::FlexBasis::auto()",
    spec="https://drafts.csswg.org/css-flexbox/#flex-basis-property",
    extra_prefixes="webkit",
    animation_value_type="FlexBasis",
    servo_restyle_damage = "reflow",
)}

% for (size, logical) in ALL_SIZES:
    <%
        spec = "https://drafts.csswg.org/css-box/#propdef-%s"
        if logical:
            spec = "https://drafts.csswg.org/css-logical-props/#propdef-%s"
    %>
    % if product == "gecko":
        // width, height, block-size, inline-size
        ${helpers.predefined_type(
            size,
            "MozLength",
            "computed::MozLength::auto()",
            logical=logical,
            logical_group="size",
            allow_quirks=not logical,
            spec=spec % size,
            animation_value_type="MozLength",
            flags="GETCS_NEEDS_LAYOUT_FLUSH",
            servo_restyle_damage="reflow",
        )}
        // min-width, min-height, min-block-size, min-inline-size,
        ${helpers.predefined_type(
            "min-%s" % size,
            "MozLength",
            "computed::MozLength::auto()",
            logical=logical,
            logical_group="min-size",
            allow_quirks=not logical,
            spec=spec % size,
            animation_value_type="MozLength",
            servo_restyle_damage="reflow",
        )}
        ${helpers.predefined_type(
            "max-%s" % size,
            "MaxLength",
            "computed::MaxLength::none()",
            logical=logical,
            logical_group="max-size",
            allow_quirks=not logical,
            spec=spec % size,
            animation_value_type="MaxLength",
            servo_restyle_damage="reflow",
        )}
    % else:
        // servo versions (no keyword support)
        ${helpers.predefined_type(
            size,
            "LengthPercentageOrAuto",
            "computed::LengthPercentageOrAuto::Auto",
            "parse_non_negative",
            spec=spec % size,
            logical_group="size",
            allow_quirks=not logical,
            animation_value_type="ComputedValue", logical = logical,
            servo_restyle_damage="reflow",
        )}
        ${helpers.predefined_type(
            "min-%s" % size,
            "LengthPercentage",
            "computed::LengthPercentage::zero()",
            "parse_non_negative",
            spec=spec % ("min-%s" % size),
            logical_group="min-size",
            animation_value_type="ComputedValue",
            logical=logical,
            allow_quirks=not logical,
            servo_restyle_damage="reflow",
        )}
        ${helpers.predefined_type(
            "max-%s" % size,
            "LengthPercentageOrNone",
            "computed::LengthPercentageOrNone::None",
            "parse_non_negative",
            spec=spec % ("max-%s" % size),
            logical_group="max-size",
            animation_value_type="ComputedValue",
            logical=logical,
            allow_quirks=not logical,
            servo_restyle_damage="reflow",
        )}
    % endif
% endfor

${helpers.single_keyword(
    "box-sizing",
    "content-box border-box",
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
    products="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-images/#propdef-object-fit",
)}

${helpers.predefined_type(
    "object-position",
    "Position",
    "computed::Position::zero()",
    products="gecko",
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
            animation_value_type="discrete",
            spec="https://drafts.csswg.org/css-grid/#propdef-grid-%s-%s" % (kind, range),
            products="gecko",
            boxed=True,
        )}
    % endfor

    // NOTE: According to the spec, this should handle multiple values of `<track-size>`,
    // but gecko supports only a single value
    ${helpers.predefined_type(
        "grid-auto-%ss" % kind,
        "TrackSize",
        "Default::default()",
        animation_value_type="discrete",
        spec="https://drafts.csswg.org/css-grid/#propdef-grid-auto-%ss" % kind,
        products="gecko",
        boxed=True,
    )}

    ${helpers.predefined_type(
        "grid-template-%ss" % kind,
        "GridTemplateComponent",
        "specified::GenericGridTemplateComponent::None",
        products="gecko",
        spec="https://drafts.csswg.org/css-grid/#propdef-grid-template-%ss" % kind,
        boxed=True,
        flags="GETCS_NEEDS_LAYOUT_FLUSH",
        animation_value_type="discrete",
    )}

% endfor

${helpers.predefined_type(
    "grid-auto-flow",
    "GridAutoFlow",
    "computed::GridAutoFlow::row()",
    products="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-grid/#propdef-grid-auto-flow",
)}

${helpers.predefined_type(
    "grid-template-areas",
    "GridTemplateAreas",
    "computed::GridTemplateAreas::none()",
    products="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-grid/#propdef-grid-template-areas",
)}

${helpers.predefined_type(
    "column-gap",
    "length::NonNegativeLengthPercentageOrNormal",
    "Either::Second(Normal)",
    alias="grid-column-gap" if product == "gecko" else "",
    extra_prefixes="moz",
    servo_pref="layout.columns.enabled",
    spec="https://drafts.csswg.org/css-align-3/#propdef-column-gap",
    animation_value_type="NonNegativeLengthPercentageOrNormal",
    servo_restyle_damage="reflow",
)}

// no need for -moz- prefixed alias for this property
${helpers.predefined_type(
    "row-gap",
    "length::NonNegativeLengthPercentageOrNormal",
    "Either::Second(Normal)",
    alias="grid-row-gap",
    products="gecko",
    spec="https://drafts.csswg.org/css-align-3/#propdef-row-gap",
    animation_value_type="NonNegativeLengthPercentageOrNormal",
    servo_restyle_damage="reflow",
)}
