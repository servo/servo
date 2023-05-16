/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// Box-shadow, etc.
<% data.new_style_struct("Effects", inherited=False) %>

${helpers.predefined_type(
    "opacity",
    "Opacity",
    "1.0",
    engines="gecko servo-2013 servo-2020",
    animation_value_type="ComputedValue",
    flags="CAN_ANIMATE_ON_COMPOSITOR",
    spec="https://drafts.csswg.org/css-color/#transparency",
    servo_restyle_damage = "reflow_out_of_flow",
)}

${helpers.predefined_type(
    "box-shadow",
    "BoxShadow",
    None,
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    vector=True,
    simple_vector_bindings=True,
    animation_value_type="AnimatedBoxShadowList",
    vector_animation_type="with_zero",
    extra_prefixes="webkit",
    ignored_when_colors_disabled=True,
    spec="https://drafts.csswg.org/css-backgrounds/#box-shadow",
)}

${helpers.predefined_type(
    "clip",
    "ClipRectOrAuto",
    "computed::ClipRectOrAuto::auto()",
    engines="gecko servo-2013 servo-2020",
    animation_value_type="ComputedValue",
    boxed=True,
    allow_quirks="Yes",
    spec="https://drafts.fxtf.org/css-masking/#clip-property",
)}

${helpers.predefined_type(
    "filter",
    "Filter",
    None,
    engines="gecko servo-2013 servo-2020",
    vector=True,
    simple_vector_bindings=True,
    gecko_ffi_name="mFilters",
    separator="Space",
    animation_value_type="AnimatedFilterList",
    vector_animation_type="with_zero",
    extra_prefixes="webkit",
    spec="https://drafts.fxtf.org/filters/#propdef-filter",
)}

${helpers.predefined_type(
    "backdrop-filter",
    "Filter",
    None,
    engines="gecko",
    vector=True,
    simple_vector_bindings=True,
    gecko_ffi_name="mBackdropFilters",
    separator="Space",
    animation_value_type="AnimatedFilterList",
    vector_animation_type="with_zero",
    gecko_pref="layout.css.backdrop-filter.enabled",
    spec="https://drafts.fxtf.org/filter-effects-2/#propdef-backdrop-filter",
)}

${helpers.single_keyword(
    "mix-blend-mode",
    """normal multiply screen overlay darken lighten color-dodge
    color-burn hard-light soft-light difference exclusion hue
    saturation color luminosity""",
    engines="gecko servo-2013 servo-2020",
    gecko_enum_prefix="StyleBlend",
    animation_value_type="discrete",
    spec="https://drafts.fxtf.org/compositing/#propdef-mix-blend-mode",
)}
