/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// Box-shadow, etc.
<% data.new_style_struct("Effects", inherited=False) %>

${helpers.predefined_type("opacity",
                          "Opacity",
                          "1.0",
                          animation_value_type="ComputedValue",
                          flags="CREATES_STACKING_CONTEXT",
                          spec="https://drafts.csswg.org/css-color/#opacity")}

<%helpers:vector_longhand name="box-shadow" allow_empty="True"
                          animation_value_type="IntermediateShadowList"
                          extra_prefixes="webkit"
                          ignored_when_colors_disabled="True"
                          spec="https://drafts.csswg.org/css-backgrounds/#box-shadow">
    pub type SpecifiedValue = specified::Shadow;

    pub mod computed_value {
        use values::computed::Shadow;

        pub type T = Shadow;
    }

    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<specified::Shadow, ParseError<'i>> {
        specified::Shadow::parse(context, input, false)
    }
</%helpers:vector_longhand>

${helpers.predefined_type("clip",
                          "ClipRectOrAuto",
                          "computed::ClipRectOrAuto::auto()",
                          animation_value_type="ComputedValue",
                          boxed="True",
                          allow_quirks=True,
                          spec="https://drafts.fxtf.org/css-masking/#clip-property")}

${helpers.predefined_type(
    "filter",
    "Filter",
    None,
    vector=True,
    separator="Space",
    animation_value_type="AnimatedFilterList",
    extra_prefixes="webkit",
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB",
    spec="https://drafts.fxtf.org/filters/#propdef-filter",
)}

${helpers.single_keyword("mix-blend-mode",
                         """normal multiply screen overlay darken lighten color-dodge
                            color-burn hard-light soft-light difference exclusion hue
                            saturation color luminosity""", gecko_constant_prefix="NS_STYLE_BLEND",
                         animation_value_type="discrete",
                         flags="CREATES_STACKING_CONTEXT",
                         spec="https://drafts.fxtf.org/compositing/#propdef-mix-blend-mode")}
