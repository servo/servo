/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// `data` comes from components/style/properties.mako.rs; see build.rs for more details.

<%!
    from data import to_rust_ident, to_camel_case
    from data import Keyword
%>
<%namespace name="helpers" file="/helpers.mako.rs" />

use app_units::Au;
use custom_properties::CustomPropertiesMap;
use gecko_bindings::bindings;
% for style_struct in data.style_structs:
use gecko_bindings::structs::${style_struct.gecko_ffi_name};
use gecko_bindings::bindings::Gecko_Construct_Default_${style_struct.gecko_ffi_name};
use gecko_bindings::bindings::Gecko_CopyConstruct_${style_struct.gecko_ffi_name};
use gecko_bindings::bindings::Gecko_Destroy_${style_struct.gecko_ffi_name};
% endfor
use gecko_bindings::bindings::Gecko_Construct_nsStyleVariables;
use gecko_bindings::bindings::Gecko_CopyCounterStyle;
use gecko_bindings::bindings::Gecko_CopyCursorArrayFrom;
use gecko_bindings::bindings::Gecko_CopyFontFamilyFrom;
use gecko_bindings::bindings::Gecko_CopyImageValueFrom;
use gecko_bindings::bindings::Gecko_CopyListStyleImageFrom;
use gecko_bindings::bindings::Gecko_Destroy_nsStyleVariables;
use gecko_bindings::bindings::Gecko_EnsureImageLayersLength;
use gecko_bindings::bindings::Gecko_FontFamilyList_AppendGeneric;
use gecko_bindings::bindings::Gecko_FontFamilyList_AppendNamed;
use gecko_bindings::bindings::Gecko_FontFamilyList_Clear;
use gecko_bindings::bindings::Gecko_SetCursorArrayLength;
use gecko_bindings::bindings::Gecko_SetCursorImageValue;
use gecko_bindings::bindings::Gecko_StyleTransition_SetUnsupportedProperty;
use gecko_bindings::bindings::Gecko_NewCSSShadowArray;
use gecko_bindings::bindings::Gecko_nsStyleFont_SetLang;
use gecko_bindings::bindings::Gecko_nsStyleFont_CopyLangFrom;
use gecko_bindings::bindings::Gecko_SetListStyleImageNone;
use gecko_bindings::bindings::Gecko_SetListStyleImageImageValue;
use gecko_bindings::bindings::Gecko_SetNullImageValue;
use gecko_bindings::bindings::ServoComputedValuesBorrowedOrNull;
use gecko_bindings::bindings::{Gecko_ResetFilters, Gecko_CopyFiltersFrom};
use gecko_bindings::bindings::RawGeckoPresContextBorrowed;
use gecko_bindings::structs;
use gecko_bindings::structs::nsCSSPropertyID;
use gecko_bindings::structs::nsStyleVariables;
use gecko_bindings::sugar::ns_style_coord::{CoordDataValue, CoordData, CoordDataMut};
use gecko_bindings::sugar::ownership::HasArcFFI;
use gecko::values::convert_nscolor_to_rgba;
use gecko::values::convert_rgba_to_nscolor;
use gecko::values::GeckoStyleCoordConvertible;
use gecko::values::round_border_to_device_pixels;
use logical_geometry::WritingMode;
use media_queries::Device;
use properties::animated_properties::TransitionProperty;
use properties::computed_value_flags::ComputedValueFlags;
use properties::{longhands, FontComputationData, Importance, LonghandId};
use properties::{PropertyDeclaration, PropertyDeclarationBlock, PropertyDeclarationId};
use rule_tree::StrongRuleNode;
use std::fmt::{self, Debug};
use std::mem::{forget, transmute, zeroed};
use std::ptr;
use stylearc::Arc;
use std::cmp;
use values::{Auto, CustomIdent, Either, KeyframesName};
use values::computed::ToComputedValue;
use values::computed::effects::{BoxShadow, Filter, SimpleShadow};
use values::specified::length::Percentage;
use computed_values::border_style;

pub mod style_structs {
    % for style_struct in data.style_structs:
    pub use super::${style_struct.gecko_struct_name} as ${style_struct.name};
    % endfor
}

// FIXME(emilio): Unify both definitions, since they're equal now.
#[derive(Clone)]
pub struct ComputedValues {
    % for style_struct in data.style_structs:
    ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
    % endfor
    custom_properties: Option<Arc<CustomPropertiesMap>>,
    pub writing_mode: WritingMode,
    pub font_computation_data: FontComputationData,
    pub flags: ComputedValueFlags,

    /// The rule node representing the ordered list of rules matched for this
    /// node.  Can be None for default values and text nodes.  This is
    /// essentially an optimization to avoid referencing the root rule node.
    pub rules: Option<StrongRuleNode>,
    /// The element's computed values if visited, only computed if there's a
    /// relevant link for this element. A element's "relevant link" is the
    /// element being matched if it is a link or the nearest ancestor link.
    visited_style: Option<Arc<ComputedValues>>,
}

impl ComputedValues {
    pub fn new(custom_properties: Option<Arc<CustomPropertiesMap>>,
               writing_mode: WritingMode,
               font_size_keyword: Option<(longhands::font_size::KeywordSize, f32)>,
               flags: ComputedValueFlags,
               rules: Option<StrongRuleNode>,
               visited_style: Option<Arc<ComputedValues>>,
               % for style_struct in data.style_structs:
               ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
               % endfor
    ) -> Self {
        ComputedValues {
            custom_properties,
            writing_mode,
            font_computation_data: FontComputationData::new(font_size_keyword),
            flags,
            rules,
            visited_style: visited_style,
            % for style_struct in data.style_structs:
            ${style_struct.ident},
            % endfor
        }
    }

    pub fn default_values(pres_context: RawGeckoPresContextBorrowed) -> Arc<Self> {
        Arc::new(ComputedValues {
            custom_properties: None,
            writing_mode: WritingMode::empty(), // FIXME(bz): This seems dubious
            font_computation_data: FontComputationData::default_values(),
            flags: ComputedValueFlags::initial(),
            rules: None,
            visited_style: None,
            % for style_struct in data.style_structs:
                ${style_struct.ident}: style_structs::${style_struct.name}::default(pres_context),
            % endfor
        })
    }

    #[inline]
    pub fn is_display_contents(&self) -> bool {
        self.get_box().clone_display() == longhands::display::computed_value::T::contents
    }

    /// Returns true if the value of the `content` property would make a
    /// pseudo-element not rendered.
    #[inline]
    pub fn ineffective_content_property(&self) -> bool {
        self.get_counters().ineffective_content_property()
    }

    % for style_struct in data.style_structs:
    #[inline]
    pub fn clone_${style_struct.name_lower}(&self) -> Arc<style_structs::${style_struct.name}> {
        self.${style_struct.ident}.clone()
    }
    #[inline]
    pub fn get_${style_struct.name_lower}(&self) -> &style_structs::${style_struct.name} {
        &self.${style_struct.ident}
    }

    pub fn ${style_struct.name_lower}_arc(&self) -> &Arc<style_structs::${style_struct.name}> {
        &self.${style_struct.ident}
    }

    #[inline]
    pub fn mutate_${style_struct.name_lower}(&mut self) -> &mut style_structs::${style_struct.name} {
        Arc::make_mut(&mut self.${style_struct.ident})
    }
    % endfor

    /// Gets a reference to the rule node. Panic if no rule node exists.
    pub fn rules(&self) -> &StrongRuleNode {
        self.rules.as_ref().unwrap()
    }

    /// Whether there is a visited style.
    pub fn has_visited_style(&self) -> bool {
        self.visited_style.is_some()
    }

    /// Gets a reference to the visited style, if any.
    pub fn get_visited_style(&self) -> Option<<&Arc<ComputedValues>> {
        self.visited_style.as_ref()
    }

    /// Gets a reference to the visited style. Panic if no visited style exists.
    pub fn visited_style(&self) -> &Arc<ComputedValues> {
        self.get_visited_style().unwrap()
    }

    /// Clone the visited style.  Used for inheriting parent styles in
    /// StyleBuilder::for_inheritance.
    pub fn clone_visited_style(&self) -> Option<Arc<ComputedValues>> {
        self.visited_style.clone()
    }

    pub fn custom_properties(&self) -> Option<Arc<CustomPropertiesMap>> {
        self.custom_properties.clone()
    }

    #[allow(non_snake_case)]
    pub fn has_moz_binding(&self) -> bool {
        !self.get_box().gecko.mBinding.mPtr.mRawPtr.is_null()
    }

    // FIXME(bholley): Implement this properly.
    #[inline]
    pub fn is_multicol(&self) -> bool { false }

    pub fn to_declaration_block(&self, property: PropertyDeclarationId) -> PropertyDeclarationBlock {
        match property {
            % for prop in data.longhands:
                % if prop.animatable:
                    PropertyDeclarationId::Longhand(LonghandId::${prop.camel_case}) => {
                         PropertyDeclarationBlock::with_one(
                            PropertyDeclaration::${prop.camel_case}(
                                % if prop.boxed:
                                    Box::new(
                                % endif
                                longhands::${prop.ident}::SpecifiedValue::from_computed_value(
                                  &self.get_${prop.style_struct.ident.strip("_")}().clone_${prop.ident}())
                                % if prop.boxed:
                                    )
                                % endif
                            ),
                            Importance::Normal
                        )
                    },
                % endif
            % endfor
            PropertyDeclarationId::Custom(_name) => unimplemented!(),
            _ => unimplemented!()
        }
    }
}

<%def name="declare_style_struct(style_struct)">
pub struct ${style_struct.gecko_struct_name} {
    gecko: ${style_struct.gecko_ffi_name},
}
impl ${style_struct.gecko_struct_name} {
    pub fn gecko(&self) -> &${style_struct.gecko_ffi_name} {
        &self.gecko
    }
    pub fn gecko_mut(&mut self) -> &mut ${style_struct.gecko_ffi_name} {
        &mut self.gecko
    }
}
</%def>

<%def name="impl_simple_setter(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        ${set_gecko_property(gecko_ffi_name, "v")}
    }
</%def>

<%def name="impl_simple_clone(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        self.gecko.${gecko_ffi_name}
    }
</%def>

<%def name="impl_simple_copy(ident, gecko_ffi_name, *kwargs)">
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name} = other.gecko.${gecko_ffi_name};
    }
</%def>

<%def name="impl_coord_copy(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name}.copy_from(&other.gecko.${gecko_ffi_name});
    }
</%def>

<%!
def get_gecko_property(ffi_name, self_param = "self"):
    if "mBorderColor" in ffi_name:
        return ffi_name.replace("mBorderColor",
                                "unsafe { *%s.gecko.__bindgen_anon_1.mBorderColor.as_ref() }"
                                % self_param)
    return "%s.gecko.%s" % (self_param, ffi_name)

def set_gecko_property(ffi_name, expr):
    if "mBorderColor" in ffi_name:
        ffi_name = ffi_name.replace("mBorderColor",
                                    "*self.gecko.__bindgen_anon_1.mBorderColor.as_mut()")
        return "unsafe { %s = %s };" % (ffi_name, expr)
    return "self.gecko.%s = %s;" % (ffi_name, expr)
%>

<%def name="impl_keyword_setter(ident, gecko_ffi_name, keyword, cast_type='u8', on_set=None)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        use properties::longhands::${ident}::computed_value::T as Keyword;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        let result = match v {
            % for value in keyword.values_for('gecko'):
                Keyword::${to_rust_ident(value)} =>
                    structs::${keyword.gecko_constant(value)} ${keyword.maybe_cast(cast_type)},
            % endfor
        };
        ${set_gecko_property(gecko_ffi_name, "result")}
        % if on_set:
        self.${on_set}();
        % endif
    }
</%def>

<%def name="impl_keyword_clone(ident, gecko_ffi_name, keyword, cast_type='u8')">
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use properties::longhands::${ident}::computed_value::T as Keyword;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts

        // Some constant macros in the gecko are defined as negative integer(e.g. font-stretch).
        // And they are convert to signed integer in Rust bindings. We need to cast then
        // as signed type when we have both signed/unsigned integer in order to use them
        // as match's arms.
        // Also, to use same implementation here we use casted constant if we have only singed values.
        % for value in keyword.values_for('gecko'):
        const ${keyword.casted_constant_name(value, cast_type)} : ${cast_type} =
            structs::${keyword.gecko_constant(value)} as ${cast_type};
        % endfor

        match ${get_gecko_property(gecko_ffi_name)} as ${cast_type} {
            % for value in keyword.values_for('gecko'):
            ${keyword.casted_constant_name(value, cast_type)} => Keyword::${to_rust_ident(value)},
            % endfor
            % if keyword.gecko_inexhaustive:
            x => panic!("Found unexpected value in style struct for ${ident} property: {:?}", x),
            % endif
        }
    }
</%def>

<%def name="impl_color_setter(ident, gecko_ffi_name)">
    #[allow(unreachable_code)]
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        ${set_gecko_property(gecko_ffi_name, "v.into()")}
    }
</%def>

<%def name="impl_color_copy(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        let color = ${get_gecko_property(gecko_ffi_name, self_param = "other")};
        ${set_gecko_property(gecko_ffi_name, "color")};
    }
</%def>

<%def name="impl_color_clone(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        ${get_gecko_property(gecko_ffi_name)}.into()
    }
</%def>

<%def name="impl_keyword(ident, gecko_ffi_name, keyword, need_clone, cast_type='u8', **kwargs)">
<%call expr="impl_keyword_setter(ident, gecko_ffi_name, keyword, cast_type, **kwargs)"></%call>
<%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
%if need_clone:
<%call expr="impl_keyword_clone(ident, gecko_ffi_name, keyword, cast_type)"></%call>
% endif
</%def>

<%def name="impl_simple(ident, gecko_ffi_name, need_clone=False)">
<%call expr="impl_simple_setter(ident, gecko_ffi_name)"></%call>
<%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
% if need_clone:
    <%call expr="impl_simple_clone(ident, gecko_ffi_name)"></%call>
% endif
</%def>

<%def name="impl_absolute_length(ident, gecko_ffi_name, need_clone=False)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        ${set_gecko_property(gecko_ffi_name, "v.0")}
    }
    <%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
    % if need_clone:
        #[allow(non_snake_case)]
        pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
            Au(self.gecko.${gecko_ffi_name})
        }
    % endif
</%def>

<%def name="impl_position(ident, gecko_ffi_name, need_clone=False)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        ${set_gecko_property("%s.mXPosition" % gecko_ffi_name, "v.horizontal.into()")}
        ${set_gecko_property("%s.mYPosition" % gecko_ffi_name, "v.vertical.into()")}
    }
    <%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
    % if need_clone:
        #[allow(non_snake_case)]
        pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
            longhands::${ident}::computed_value::T {
                horizontal: self.gecko.${gecko_ffi_name}.mXPosition.into(),
                vertical: self.gecko.${gecko_ffi_name}.mYPosition.into(),
            }
        }
    % endif
</%def>

<%def name="impl_color(ident, gecko_ffi_name, need_clone=False)">
<%call expr="impl_color_setter(ident, gecko_ffi_name)"></%call>
<%call expr="impl_color_copy(ident, gecko_ffi_name)"></%call>
% if need_clone:
    <%call expr="impl_color_clone(ident, gecko_ffi_name)"></%call>
% endif
</%def>

<%def name="impl_rgba_color(ident, gecko_ffi_name, need_clone=False)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        ${set_gecko_property(gecko_ffi_name, "convert_rgba_to_nscolor(&v)")}
    }
    <%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
    % if need_clone:
        #[allow(non_snake_case)]
        pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
            convert_nscolor_to_rgba(${get_gecko_property(gecko_ffi_name)})
        }
    % endif
</%def>

<%def name="impl_svg_paint(ident, gecko_ffi_name, need_clone=False)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, mut v: longhands::${ident}::computed_value::T) {
        use values::generics::SVGPaintKind;
        use self::structs::nsStyleSVGPaintType;
        use self::structs::nsStyleSVGFallbackType;

        let ref mut paint = ${get_gecko_property(gecko_ffi_name)};
        unsafe {
            bindings::Gecko_nsStyleSVGPaint_Reset(paint);
        }
        let fallback = v.fallback.take();
        match v.kind {
            SVGPaintKind::None => return,
            SVGPaintKind::ContextFill => {
                paint.mType = nsStyleSVGPaintType::eStyleSVGPaintType_ContextFill;
            }
            SVGPaintKind::ContextStroke => {
                paint.mType = nsStyleSVGPaintType::eStyleSVGPaintType_ContextStroke;
            }
            SVGPaintKind::PaintServer(url) => {
                unsafe {
                    bindings::Gecko_nsStyleSVGPaint_SetURLValue(paint, url.for_ffi());
                }
            }
            SVGPaintKind::Color(color) => {
                paint.mType = nsStyleSVGPaintType::eStyleSVGPaintType_Color;
                unsafe {
                    *paint.mPaint.mColor.as_mut() = convert_rgba_to_nscolor(&color);
                }
            }
        }

        if let Some(fallback) = fallback {
            paint.mFallbackType = nsStyleSVGFallbackType::eStyleSVGFallbackType_Color;
            paint.mFallbackColor = convert_rgba_to_nscolor(&fallback);
        }
    }

    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        unsafe {
            bindings::Gecko_nsStyleSVGPaint_CopyFrom(
                &mut ${get_gecko_property(gecko_ffi_name)},
                & ${get_gecko_property(gecko_ffi_name, "other")}
            );
        }
    }

    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use values::generics::{SVGPaint, SVGPaintKind};
        use values::specified::url::SpecifiedUrl;
        use self::structs::nsStyleSVGPaintType;
        use self::structs::nsStyleSVGFallbackType;
        let ref paint = ${get_gecko_property(gecko_ffi_name)};
        let fallback = if let nsStyleSVGFallbackType::eStyleSVGFallbackType_Color = paint.mFallbackType {
            Some(convert_nscolor_to_rgba(paint.mFallbackColor))
        } else {
            None
        };
        let kind = match paint.mType {
            nsStyleSVGPaintType::eStyleSVGPaintType_None => SVGPaintKind::None,
            nsStyleSVGPaintType::eStyleSVGPaintType_ContextFill => SVGPaintKind::ContextFill,
            nsStyleSVGPaintType::eStyleSVGPaintType_ContextStroke => SVGPaintKind::ContextStroke,
            nsStyleSVGPaintType::eStyleSVGPaintType_Server => {
                unsafe {
                    SVGPaintKind::PaintServer(
                        SpecifiedUrl::from_url_value_data(
                            &(**paint.mPaint.mPaintServer.as_ref())._base
                        ).unwrap()
                    )
                }
            }
            nsStyleSVGPaintType::eStyleSVGPaintType_Color => {
                unsafe { SVGPaintKind::Color(convert_nscolor_to_rgba(*paint.mPaint.mColor.as_ref())) }
            }
        };
        SVGPaint {
            kind: kind,
            fallback: fallback,
        }
    }
</%def>

<%def name="impl_app_units(ident, gecko_ffi_name, need_clone, inherit_from=None, round_to_pixels=False)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        let value = {
            % if round_to_pixels:
            let au_per_device_px = Au(self.gecko.mTwipsPerPixel);
            round_border_to_device_pixels(v, au_per_device_px).0
            % else:
            v.0
            % endif
        };

        % if inherit_from:
        self.gecko.${inherit_from} = value;
        % endif
        self.gecko.${gecko_ffi_name} = value;
    }

    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        % if inherit_from:
        self.gecko.${inherit_from} = other.gecko.${inherit_from};
        // NOTE: This is needed to easily handle the `unset` and `initial`
        // keywords, which are implemented calling this function.
        //
        // In practice, this means that we may have an incorrect value here, but
        // we'll adjust that properly in the style fixup phase.
        self.gecko.${gecko_ffi_name} = other.gecko.${inherit_from};
        % else:
        self.gecko.${gecko_ffi_name} = other.gecko.${gecko_ffi_name};
        % endif
    }

%if need_clone:
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        Au(self.gecko.${gecko_ffi_name})
    }
% endif
</%def>

<%def name="impl_split_style_coord(ident, gecko_ffi_name, index, need_clone=False)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        v.to_gecko_style_coord(&mut self.gecko.${gecko_ffi_name}.data_at_mut(${index}));
    }
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name}.data_at_mut(${index}).copy_from(&other.gecko.${gecko_ffi_name}.data_at(${index}));
    }
    % if need_clone:
        #[allow(non_snake_case)]
        pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
            use properties::longhands::${ident}::computed_value::T;
            T::from_gecko_style_coord(&self.gecko.${gecko_ffi_name}.data_at(${index}))
                .expect("clone for ${ident} failed")
        }
    % endif
</%def>

<%def name="impl_style_coord(ident, gecko_ffi_name, need_clone=False)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        v.to_gecko_style_coord(&mut self.gecko.${gecko_ffi_name});
    }
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name}.copy_from(&other.gecko.${gecko_ffi_name});
    }
    % if need_clone:
        #[allow(non_snake_case)]
        pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
            use properties::longhands::${ident}::computed_value::T;
            T::from_gecko_style_coord(&self.gecko.${gecko_ffi_name})
                .expect("clone for ${ident} failed")
        }
    % endif
</%def>

<%def name="impl_style_sides(ident)">
    <% gecko_ffi_name = "m" + to_camel_case(ident) %>

    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        v.to_gecko_rect(&mut self.gecko.${gecko_ffi_name});
    }

    <%self:copy_sides_style_coord ident="${ident}"></%self:copy_sides_style_coord>

    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        longhands::${ident}::computed_value::T::from_gecko_rect(&self.gecko.${gecko_ffi_name})
            .expect("clone for ${ident} failed")
    }
</%def>

<%def name="copy_sides_style_coord(ident)">
    <% gecko_ffi_name = "m" + to_camel_case(ident) %>
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        % for side in SIDES:
            self.gecko.${gecko_ffi_name}.data_at_mut(${side.index})
                .copy_from(&other.gecko.${gecko_ffi_name}.data_at(${side.index}));
        % endfor
        ${ caller.body() }
    }
</%def>

<%def name="impl_corner_style_coord(ident, gecko_ffi_name, x_index, y_index, need_clone)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        v.0.width.to_gecko_style_coord(&mut self.gecko.${gecko_ffi_name}.data_at_mut(${x_index}));
        v.0.height.to_gecko_style_coord(&mut self.gecko.${gecko_ffi_name}.data_at_mut(${y_index}));
    }
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name}.data_at_mut(${x_index})
                  .copy_from(&other.gecko.${gecko_ffi_name}.data_at(${x_index}));
        self.gecko.${gecko_ffi_name}.data_at_mut(${y_index})
                  .copy_from(&other.gecko.${gecko_ffi_name}.data_at(${y_index}));
    }
    % if need_clone:
        #[allow(non_snake_case)]
        pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
            use values::computed::border::BorderCornerRadius;
            let width = GeckoStyleCoordConvertible::from_gecko_style_coord(
                            &self.gecko.${gecko_ffi_name}.data_at(${x_index}))
                            .expect("Failed to clone ${ident}");
            let height = GeckoStyleCoordConvertible::from_gecko_style_coord(
                            &self.gecko.${gecko_ffi_name}.data_at(${y_index}))
                            .expect("Failed to clone ${ident}");
            BorderCornerRadius::new(width, height)
        }
    % endif
</%def>

<%def name="impl_css_url(ident, gecko_ffi_name, need_clone=False)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        use gecko_bindings::sugar::refptr::RefPtr;
        match v {
            Either::First(url) => {
                let refptr = unsafe {
                    let ptr = bindings::Gecko_NewURLValue(url.for_ffi());
                    if ptr.is_null() {
                        self.gecko.${gecko_ffi_name}.clear();
                        return;
                    }
                    RefPtr::from_addrefed(ptr)
                };
                self.gecko.${gecko_ffi_name}.set_move(refptr)
            }
            Either::Second(_none) => {
                unsafe {
                    self.gecko.${gecko_ffi_name}.clear();
                }
            }
        }
    }
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        unsafe {
            self.gecko.${gecko_ffi_name}.set(&other.gecko.${gecko_ffi_name});
        }
    }
    % if need_clone:
        pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
            use values::specified::url::SpecifiedUrl;
            use values::None_;

            if self.gecko.${gecko_ffi_name}.mRawPtr.is_null() {
                Either::Second(None_)
            } else {
                unsafe {
                    let ref gecko_url_value = *self.gecko.${gecko_ffi_name}.mRawPtr;
                    Either::First(SpecifiedUrl::from_url_value_data(&gecko_url_value._base)
                            .expect("${gecko_ffi_name} could not convert to SpecifiedUrl"))
                }
            }
        }
    % endif
</%def>

<%def name="impl_logical(name, need_clone=False, **kwargs)">
    ${helpers.logical_setter(name, need_clone)}
</%def>

<%def name="impl_style_struct(style_struct)">
impl ${style_struct.gecko_struct_name} {
    #[allow(dead_code, unused_variables)]
    pub fn default(pres_context: RawGeckoPresContextBorrowed) -> Arc<Self> {
        let mut result = Arc::new(${style_struct.gecko_struct_name} { gecko: unsafe { zeroed() } });
        unsafe {
            Gecko_Construct_Default_${style_struct.gecko_ffi_name}(&mut Arc::get_mut(&mut result).unwrap().gecko,
                                                                   pres_context);
        }
        result
    }
    pub fn get_gecko(&self) -> &${style_struct.gecko_ffi_name} {
        &self.gecko
    }
}
impl Drop for ${style_struct.gecko_struct_name} {
    fn drop(&mut self) {
        unsafe {
            Gecko_Destroy_${style_struct.gecko_ffi_name}(&mut self.gecko);
        }
    }
}
impl Clone for ${style_struct.gecko_struct_name} {
    fn clone(&self) -> Self {
        unsafe {
            let mut result = ${style_struct.gecko_struct_name} { gecko: zeroed() };
            Gecko_CopyConstruct_${style_struct.gecko_ffi_name}(&mut result.gecko, &self.gecko);
            result
        }
    }
}

// FIXME(bholley): Make bindgen generate Debug for all types.
%if style_struct.gecko_ffi_name in ("nsStyle" + x for x in "Border Display List Background Font SVGReset".split()):
impl Debug for ${style_struct.gecko_struct_name} {
    // FIXME(bholley): Generate this.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Gecko style struct: ${style_struct.gecko_struct_name}")
    }
}
%else:
impl Debug for ${style_struct.gecko_struct_name} {
    // FIXME(bholley): Generate this.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.gecko.fmt(f) }
}
%endif
</%def>

<%def name="impl_simple_type_with_conversion(ident, gecko_ffi_name=None)">
    <%
    if gecko_ffi_name is None:
        gecko_ffi_name = "m" + to_camel_case(ident)
    %>

    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        self.gecko.${gecko_ffi_name} = From::from(v)
    }

    <% impl_simple_copy(ident, gecko_ffi_name) %>

    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        From::from(self.gecko.${gecko_ffi_name})
    }
</%def>

<%def name="raw_impl_trait(style_struct, skip_longhands='', skip_additionals='')">
<%
    longhands = [x for x in style_struct.longhands
                if not (skip_longhands == "*" or x.name in skip_longhands.split())]

    #
    # Make a list of types we can't auto-generate.
    #
    force_stub = [];
    # These have unusual representations in gecko.
    force_stub += ["list-style-type"]

    # Types used with predefined_type()-defined properties that we can auto-generate.
    predefined_types = {
        "length::LengthOrAuto": impl_style_coord,
        "length::LengthOrNormal": impl_style_coord,
        "Length": impl_absolute_length,
        "Position": impl_position,
        "LengthOrPercentage": impl_style_coord,
        "LengthOrPercentageOrAuto": impl_style_coord,
        "LengthOrPercentageOrNone": impl_style_coord,
        "LengthOrNone": impl_style_coord,
        "LengthOrNormal": impl_style_coord,
        "MaxLength": impl_style_coord,
        "MozLength": impl_style_coord,
        "Number": impl_simple,
        "Integer": impl_simple,
        "Opacity": impl_simple,
        "Color": impl_color,
        "RGBAColor": impl_rgba_color,
        "SVGPaint": impl_svg_paint,
        "UrlOrNone": impl_css_url,
    }

    def longhand_method(longhand):
        args = dict(ident=longhand.ident, gecko_ffi_name=longhand.gecko_ffi_name,
                    need_clone=longhand.need_clone)

        # get the method and pass additional keyword or type-specific arguments
        if longhand.logical:
            method = impl_logical
            args.update(name=longhand.name)
        elif longhand.keyword:
            method = impl_keyword
            args.update(keyword=longhand.keyword)
            if "font" in longhand.ident:
                args.update(cast_type=longhand.cast_type)
        else:
            method = predefined_types[longhand.predefined_type]

        method(**args)

    picked_longhands, stub_longhands = [], []
    for x in longhands:
        if (x.keyword or x.predefined_type in predefined_types or x.logical) and x.name not in force_stub:
            picked_longhands.append(x)
        else:
            stub_longhands.append(x)

    # If one of the longhands is not handled
    # by either:
    # - being a keyword
    # - being a predefined longhand
    # - being a longhand with manual glue code (i.e. in skip_longhands)
    # - being generated as a stub
    #
    # then we raise an error here.
    #
    # If you hit this error, please add `product="servo"` to the longhand.
    # In case the longhand is used in a shorthand, add it to the force_stub
    # list above.

    for stub in stub_longhands:
       if stub.name not in force_stub:
           raise Exception("Don't know what to do with longhand %s in style struct %s"
                           % (stub.name,style_struct. gecko_struct_name))
%>
impl ${style_struct.gecko_struct_name} {
    /*
     * Manually-Implemented Methods.
     */
    ${caller.body().strip()}

    /*
     * Auto-Generated Methods.
     */
    <%
    for longhand in picked_longhands:
        longhand_method(longhand)
    %>

    /*
     * Stubs.
     */
    % for longhand in stub_longhands:
    #[allow(non_snake_case)]
    pub fn set_${longhand.ident}(&mut self, _: longhands::${longhand.ident}::computed_value::T) {
        warn!("stylo: Unimplemented property setter: ${longhand.name}");
    }
    #[allow(non_snake_case)]
    pub fn copy_${longhand.ident}_from(&mut self, _: &Self) {
        warn!("stylo: Unimplemented property setter: ${longhand.name}");
    }
    % if longhand.need_clone:
    #[allow(non_snake_case)]
    pub fn clone_${longhand.ident}(&self) -> longhands::${longhand.ident}::computed_value::T {
        unimplemented!()
    }
    % endif
    % if longhand.need_index:
    pub fn ${longhand.ident}_count(&self) -> usize { 0 }
    pub fn ${longhand.ident}_at(&self, _index: usize)
                                -> longhands::${longhand.ident}::computed_value::SingleComputedValue {
        unimplemented!()
    }
    % endif
    % endfor
    <% additionals = [x for x in style_struct.additional_methods
                      if skip_additionals != "*" and not x.name in skip_additionals.split()] %>
    % for additional in additionals:
    ${additional.stub()}
    % endfor
}
</%def>

<% data.manual_style_structs = [] %>
<%def name="impl_trait(style_struct_name, skip_longhands='', skip_additionals='')">
<%self:raw_impl_trait style_struct="${next(x for x in data.style_structs if x.name == style_struct_name)}"
                      skip_longhands="${skip_longhands}" skip_additionals="${skip_additionals}">
${caller.body()}
</%self:raw_impl_trait>
<% data.manual_style_structs.append(style_struct_name) %>
</%def>

<%!
class Side(object):
    def __init__(self, name, index):
        self.name = name
        self.ident = name.lower()
        self.index = index

class Corner(object):
    def __init__(self, vert, horiz, index):
        self.x_name = "HalfCorner::eCorner" + vert + horiz + "X"
        self.y_name = "HalfCorner::eCorner" + vert + horiz + "Y"
        self.ident = (vert + "_" + horiz).lower()
        self.x_index = 2 * index
        self.y_index = 2 * index + 1

class GridLine(object):
    def __init__(self, name):
        self.ident = "grid-" + name.lower()
        self.name = self.ident.replace('-', '_')
        self.gecko = "m" + to_camel_case(self.ident)

SIDES = [Side("Top", 0), Side("Right", 1), Side("Bottom", 2), Side("Left", 3)]
CORNERS = [Corner("Top", "Left", 0), Corner("Top", "Right", 1),
           Corner("Bottom", "Right", 2), Corner("Bottom", "Left", 3)]
GRID_LINES = map(GridLine, ["row-start", "row-end", "column-start", "column-end"])
%>

#[allow(dead_code)]
fn static_assert() {
    unsafe {
        % for corner in CORNERS:
        transmute::<_, [u32; ${corner.x_index}]>([1; structs::${corner.x_name} as usize]);
        transmute::<_, [u32; ${corner.y_index}]>([1; structs::${corner.y_name} as usize]);
        % endfor
    }
    // Note: using the above technique with an enum hits a rust bug when |structs| is in a different crate.
    % for side in SIDES:
    { const DETAIL: u32 = [0][(structs::Side::eSide${side.name} as usize != ${side.index}) as usize]; let _ = DETAIL; }
    % endfor
}


<% border_style_keyword = Keyword("border-style",
                                  "none solid double dotted dashed hidden groove ridge inset outset") %>

<% skip_border_longhands = " ".join(["border-{0}-{1}".format(x.ident, y)
                                     for x in SIDES
                                     for y in ["color", "style", "width"]] +
                                    ["border-{0}-radius".format(x.ident.replace("_", "-"))
                                     for x in CORNERS]) %>

<% skip_moz_border_color_longhands = " ".join("-moz-border-{0}-colors".format(x.ident)
                                              for x in SIDES) %>
<%self:impl_trait style_struct_name="Border"
                  skip_longhands="${skip_border_longhands} border-image-source border-image-outset
                                  border-image-repeat border-image-width border-image-slice
                                  ${skip_moz_border_color_longhands}"
                  skip_additionals="*">

    % for side in SIDES:
    <% impl_keyword("border_%s_style" % side.ident,
                    "mBorderStyle[%s]" % side.index,
                    border_style_keyword,
                    on_set="update_border_%s" % side.ident,
                    need_clone=True) %>

    // This is needed because the initial mComputedBorder value is set to zero.
    //
    // In order to compute stuff, we start from the initial struct, and keep
    // going down the tree applying properties.
    //
    // That means, effectively, that when we set border-style to something
    // non-hidden, we should use the initial border instead.
    //
    // Servo stores the initial border-width in the initial struct, and then
    // adjusts as needed in the fixup phase. This means that the initial struct
    // is technically not valid without fixups, and that you lose pretty much
    // any sharing of the initial struct, which is kind of unfortunate.
    //
    // Gecko has two fields for this, one that stores the "specified" border,
    // and other that stores the actual computed one. That means that when we
    // set border-style, border-width may change and we need to sync back to the
    // specified one. This is what this function does.
    //
    // Note that this doesn't impose any dependency in the order of computation
    // of the properties. This is only relevant if border-style is specified,
    // but border-width isn't. If border-width is specified at some point, the
    // two mBorder and mComputedBorder fields would be the same already.
    //
    // Once we're here, we know that we'll run style fixups, so it's fine to
    // just copy the specified border here, we'll adjust it if it's incorrect
    // later.
    fn update_border_${side.ident}(&mut self) {
        self.gecko.mComputedBorder.${side.ident} = self.gecko.mBorder.${side.ident};
    }

    <% impl_color("border_%s_color" % side.ident, "(mBorderColor)[%s]" % side.index, need_clone=True) %>

    <% impl_app_units("border_%s_width" % side.ident,
                      "mComputedBorder.%s" % side.ident,
                      inherit_from="mBorder.%s" % side.ident,
                      need_clone=True,
                      round_to_pixels=True) %>

    pub fn border_${side.ident}_has_nonzero_width(&self) -> bool {
        self.gecko.mComputedBorder.${side.ident} != 0
    }

    #[allow(non_snake_case)]
    pub fn set__moz_border_${side.ident}_colors(&mut self,
                                                v: longhands::_moz_border_${side.ident}_colors::computed_value::T) {
        match v.0 {
            None => {
                unsafe {
                    bindings::Gecko_ClearMozBorderColors(&mut self.gecko,
                                                         structs::Side::eSide${to_camel_case(side.ident)});
                }
            },
            Some(ref colors) => {
                unsafe {
                    bindings::Gecko_EnsureMozBorderColors(&mut self.gecko);
                    bindings::Gecko_ClearMozBorderColors(&mut self.gecko,
                                                         structs::Side::eSide${to_camel_case(side.ident)});
                }
                for color in colors {
                    let c = convert_rgba_to_nscolor(color);
                    unsafe {
                        bindings::Gecko_AppendMozBorderColors(&mut self.gecko,
                                                              structs::Side::eSide${to_camel_case(side.ident)},
                                                              c);
                    }
                }
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn copy__moz_border_${side.ident}_colors_from(&mut self, other: &Self) {
        unsafe {
            bindings::Gecko_CopyMozBorderColors(&mut self.gecko, &other.gecko,
                                                structs::Side::eSide${to_camel_case(side.ident)});
        }
    }

    #[allow(non_snake_case)]
    pub fn clone__moz_border_${side.ident}_colors(&self)
                                                  -> longhands::_moz_border_${side.ident}_colors::computed_value::T {
        use self::longhands::_moz_border_${side.ident}_colors::computed_value::T;

        let mut gecko_colors =
            unsafe { bindings::Gecko_GetMozBorderColors(&self.gecko,
                                                        structs::Side::eSide${to_camel_case(side.ident)}) };

        if gecko_colors.is_null() {
            return T(None);
        }

        let mut colors = Vec::new();
        loop {
            unsafe {
                colors.push(convert_nscolor_to_rgba((*gecko_colors).mColor));
                if (*gecko_colors).mNext.is_null() { break; }
                gecko_colors = (*gecko_colors).mNext;
            }
        }
        T(Some(colors))
    }
    % endfor

    % for corner in CORNERS:
    <% impl_corner_style_coord("border_%s_radius" % corner.ident,
                               "mBorderRadius",
                               corner.x_index,
                               corner.y_index,
                               need_clone=True) %>
    % endfor

    pub fn set_border_image_source(&mut self, image: longhands::border_image_source::computed_value::T) {
        unsafe {
            // Prevent leaking of the last elements we did set
            Gecko_SetNullImageValue(&mut self.gecko.mBorderImageSource);
        }

        if let Either::Second(image) = image {
            self.gecko.mBorderImageSource.set(image, &mut false)
        }
    }

    pub fn copy_border_image_source_from(&mut self, other: &Self) {
        unsafe {
            Gecko_CopyImageValueFrom(&mut self.gecko.mBorderImageSource,
                                     &other.gecko.mBorderImageSource);
        }
    }

    pub fn clone_border_image_source(&self) -> longhands::border_image_source::computed_value::T {
        use values::None_;

        match unsafe { self.gecko.mBorderImageSource.into_image() } {
            Some(image) => Either::Second(image),
            None => Either::First(None_),
        }
    }

    <% impl_style_sides("border_image_outset") %>

    <%
    border_image_repeat_keywords = ["Stretch", "Repeat", "Round", "Space"]
    %>

    pub fn set_border_image_repeat(&mut self, v: longhands::border_image_repeat::computed_value::T) {
        use properties::longhands::border_image_repeat::computed_value::RepeatKeyword;
        use gecko_bindings::structs;

        % for i, side in enumerate(["H", "V"]):
            let k = match v.${i} {
                % for keyword in border_image_repeat_keywords:
                RepeatKeyword::${keyword} => structs::NS_STYLE_BORDER_IMAGE_REPEAT_${keyword.upper()},
                % endfor
            };

            self.gecko.mBorderImageRepeat${side} = k as u8;
        % endfor
    }

    pub fn copy_border_image_repeat_from(&mut self, other: &Self) {
        self.gecko.mBorderImageRepeatH = other.gecko.mBorderImageRepeatH;
        self.gecko.mBorderImageRepeatV = other.gecko.mBorderImageRepeatV;
    }

    pub fn clone_border_image_repeat(&self) -> longhands::border_image_repeat::computed_value::T {
        use properties::longhands::border_image_repeat::computed_value::RepeatKeyword;
        use gecko_bindings::structs;

        % for side in ["H", "V"]:
        let servo_${side.lower()} = match self.gecko.mBorderImageRepeat${side} as u32 {
            % for keyword in border_image_repeat_keywords:
            structs::NS_STYLE_BORDER_IMAGE_REPEAT_${keyword.upper()} => RepeatKeyword::${keyword},
            % endfor
            x => panic!("Found unexpected value in mBorderImageRepeat${side}: {:?}", x),
        };
        % endfor
        longhands::border_image_repeat::computed_value::T(servo_h, servo_v)
    }

    <% impl_style_sides("border_image_width") %>

    pub fn set_border_image_slice(&mut self, v: longhands::border_image_slice::computed_value::T) {
        use gecko_bindings::structs::{NS_STYLE_BORDER_IMAGE_SLICE_NOFILL, NS_STYLE_BORDER_IMAGE_SLICE_FILL};

        v.offsets.to_gecko_rect(&mut self.gecko.mBorderImageSlice);

        let fill = if v.fill {
            NS_STYLE_BORDER_IMAGE_SLICE_FILL
        } else {
            NS_STYLE_BORDER_IMAGE_SLICE_NOFILL
        };
        self.gecko.mBorderImageFill = fill as u8;
    }

    <%self:copy_sides_style_coord ident="border_image_slice">
        self.gecko.mBorderImageFill = other.gecko.mBorderImageFill;
    </%self:copy_sides_style_coord>

    pub fn clone_border_image_slice(&self) -> longhands::border_image_slice::computed_value::T {
        use gecko_bindings::structs::NS_STYLE_BORDER_IMAGE_SLICE_FILL;
        use values::computed::{BorderImageSlice, NumberOrPercentage};
        type NumberOrPercentageRect = ::values::generics::rect::Rect<NumberOrPercentage>;

        BorderImageSlice {
            offsets:
                NumberOrPercentageRect::from_gecko_rect(&self.gecko.mBorderImageSlice)
                    .expect("mBorderImageSlice[${side.index}] could not convert to NumberOrPercentageRect"),
            fill: self.gecko.mBorderImageFill as u32 == NS_STYLE_BORDER_IMAGE_SLICE_FILL
        }
    }
</%self:impl_trait>

<% skip_margin_longhands = " ".join(["margin-%s" % x.ident for x in SIDES]) %>
<%self:impl_trait style_struct_name="Margin"
                  skip_longhands="${skip_margin_longhands}">

    % for side in SIDES:
    <% impl_split_style_coord("margin_%s" % side.ident,
                              "mMargin",
                              side.index,
                              need_clone=True) %>
    % endfor
</%self:impl_trait>

<% skip_padding_longhands = " ".join(["padding-%s" % x.ident for x in SIDES]) %>
<%self:impl_trait style_struct_name="Padding"
                  skip_longhands="${skip_padding_longhands}">

    % for side in SIDES:
    <% impl_split_style_coord("padding_%s" % side.ident,
                              "mPadding",
                              side.index,
                              need_clone=True) %>
    % endfor
</%self:impl_trait>

<% skip_position_longhands = " ".join(x.ident for x in SIDES + GRID_LINES) %>
<%self:impl_trait style_struct_name="Position"
                  skip_longhands="${skip_position_longhands} z-index order align-content
                                  justify-content align-self justify-self align-items
                                  justify-items grid-auto-rows grid-auto-columns grid-auto-flow
                                  grid-template-areas grid-template-rows grid-template-columns">
    % for side in SIDES:
    <% impl_split_style_coord("%s" % side.ident,
                              "mOffset",
                              side.index,
                              need_clone=True) %>
    % endfor

    pub fn set_z_index(&mut self, v: longhands::z_index::computed_value::T) {
        match v {
            Either::First(n) => self.gecko.mZIndex.set_value(CoordDataValue::Integer(n)),
            Either::Second(Auto) => self.gecko.mZIndex.set_value(CoordDataValue::Auto),
        }
    }

    pub fn copy_z_index_from(&mut self, other: &Self) {
        use gecko_bindings::structs::nsStyleUnit;
        // z-index is never a calc(). If it were, we'd be leaking here, so
        // assert that it isn't.
        debug_assert!(self.gecko.mZIndex.unit() != nsStyleUnit::eStyleUnit_Calc);
        unsafe {
            self.gecko.mZIndex.copy_from_unchecked(&other.gecko.mZIndex);
        }
    }

    pub fn clone_z_index(&self) -> longhands::z_index::computed_value::T {
        return match self.gecko.mZIndex.as_value() {
            CoordDataValue::Integer(n) => Either::First(n),
            CoordDataValue::Auto => Either::Second(Auto),
            _ => {
                debug_assert!(false);
                Either::First(0)
            }
        }
    }

    % for kind in ["align", "justify"]:
    ${impl_simple_type_with_conversion(kind + "_content")}
    ${impl_simple_type_with_conversion(kind + "_self")}
    ${impl_simple_type_with_conversion(kind + "_items")}
    % endfor

    pub fn set_order(&mut self, v: longhands::order::computed_value::T) {
        self.gecko.mOrder = v;
    }

    pub fn clone_order(&self) -> longhands::order::computed_value::T {
        self.gecko.mOrder
    }

    ${impl_simple_copy('order', 'mOrder')}

    % for value in GRID_LINES:
    pub fn set_${value.name}(&mut self, v: longhands::${value.name}::computed_value::T) {
        use gecko_bindings::structs::{nsStyleGridLine_kMinLine, nsStyleGridLine_kMaxLine};

        let ident = v.ident.as_ref().map_or(&[] as &[_], |ident| ident.0.as_slice());
        self.gecko.${value.gecko}.mLineName.assign(ident);
        self.gecko.${value.gecko}.mHasSpan = v.is_span;
        self.gecko.${value.gecko}.mInteger = v.line_num.map(|i| {
            // clamping the integer between a range
            cmp::max(nsStyleGridLine_kMinLine, cmp::min(i.value(), nsStyleGridLine_kMaxLine))
        }).unwrap_or(0);
    }

    pub fn copy_${value.name}_from(&mut self, other: &Self) {
        self.gecko.${value.gecko}.mHasSpan = other.gecko.${value.gecko}.mHasSpan;
        self.gecko.${value.gecko}.mInteger = other.gecko.${value.gecko}.mInteger;
        self.gecko.${value.gecko}.mLineName.assign(&*other.gecko.${value.gecko}.mLineName);
    }

    pub fn clone_${value.name}(&self) -> longhands::${value.name}::computed_value::T {
        use gecko_bindings::structs::{nsStyleGridLine_kMinLine, nsStyleGridLine_kMaxLine};
        use string_cache::Atom;
        use values::specified::Integer;

        longhands::${value.name}::computed_value::T {
            is_span: self.gecko.${value.gecko}.mHasSpan,
            ident: {
                let name = self.gecko.${value.gecko}.mLineName.to_string();
                if name.len() == 0 {
                    None
                } else {
                    Some(CustomIdent(Atom::from(name)))
                }
            },
            line_num:
                if self.gecko.${value.gecko}.mInteger == 0 {
                    None
                } else {
                    debug_assert!(nsStyleGridLine_kMinLine <= self.gecko.${value.gecko}.mInteger);
                    debug_assert!(self.gecko.${value.gecko}.mInteger <= nsStyleGridLine_kMaxLine);
                    Some(Integer::new(self.gecko.${value.gecko}.mInteger))
                },
        }
    }
    % endfor

    % for kind in ["rows", "columns"]:
    pub fn set_grid_auto_${kind}(&mut self, v: longhands::grid_auto_${kind}::computed_value::T) {
        v.to_gecko_style_coords(&mut self.gecko.mGridAuto${kind.title()}Min,
                                &mut self.gecko.mGridAuto${kind.title()}Max)
    }

    pub fn copy_grid_auto_${kind}_from(&mut self, other: &Self) {
        self.gecko.mGridAuto${kind.title()}Min.copy_from(&other.gecko.mGridAuto${kind.title()}Min);
        self.gecko.mGridAuto${kind.title()}Max.copy_from(&other.gecko.mGridAuto${kind.title()}Max);
    }

    pub fn clone_grid_auto_${kind}(&self) -> longhands::grid_auto_${kind}::computed_value::T {
        ::values::generics::grid::TrackSize::from_gecko_style_coords(&self.gecko.mGridAuto${kind.title()}Min,
                                                                     &self.gecko.mGridAuto${kind.title()}Max)
    }

    pub fn set_grid_template_${kind}(&mut self, v: longhands::grid_template_${kind}::computed_value::T) {
        <% self_grid = "self.gecko.mGridTemplate%s" % kind.title() %>
        use gecko_bindings::structs::{nsTArray, nsStyleGridLine_kMaxLine};
        use nsstring::nsStringRepr;
        use std::usize;
        use values::CustomIdent;
        use values::generics::grid::TrackListType::Auto;
        use values::generics::grid::{GridTemplateComponent, RepeatCount};

        #[inline]
        fn set_line_names(servo_names: &[CustomIdent], gecko_names: &mut nsTArray<nsStringRepr>) {
            unsafe {
                bindings::Gecko_ResizeTArrayForStrings(gecko_names, servo_names.len() as u32);
            }

            for (servo_name, gecko_name) in servo_names.iter().zip(gecko_names.iter_mut()) {
                gecko_name.assign(servo_name.0.as_slice());
            }
        }

        // Set defaults
        ${self_grid}.mRepeatAutoIndex = -1;
        ${self_grid}.set_mIsAutoFill(false);
        ${self_grid}.set_mIsSubgrid(false);

        let max_lines = nsStyleGridLine_kMaxLine as usize - 1;      // for accounting the final <line-names>

        match v {
            GridTemplateComponent::TrackList(track) => {
                let mut auto_idx = usize::MAX;
                let mut auto_track_size = None;
                if let Auto(idx) = track.list_type {
                    auto_idx = idx as usize;
                    let auto_repeat = track.auto_repeat.as_ref().expect("expected <auto-track-repeat> value");

                    if auto_repeat.count == RepeatCount::AutoFill {
                        ${self_grid}.set_mIsAutoFill(true);
                    }

                    ${self_grid}.mRepeatAutoIndex = idx as i16;
                    // NOTE: Gecko supports only one set of values in <auto-repeat>
                    // i.e., it can only take repeat(auto-fill, [a] 10px [b]), and no more.
                    set_line_names(&auto_repeat.line_names[0], &mut ${self_grid}.mRepeatAutoLineNameListBefore);
                    set_line_names(&auto_repeat.line_names[1], &mut ${self_grid}.mRepeatAutoLineNameListAfter);
                    auto_track_size = Some(auto_repeat.track_sizes.get(0).unwrap().clone());
                } else {
                    unsafe {
                        bindings::Gecko_ResizeTArrayForStrings(
                            &mut ${self_grid}.mRepeatAutoLineNameListBefore, 0);
                        bindings::Gecko_ResizeTArrayForStrings(
                            &mut ${self_grid}.mRepeatAutoLineNameListAfter, 0);
                    }
                }

                let mut num_values = track.values.len();
                if auto_track_size.is_some() {
                    num_values += 1;
                }

                num_values = cmp::min(num_values, max_lines);
                unsafe {
                    bindings::Gecko_SetStyleGridTemplateArrayLengths(&mut ${self_grid}, num_values as u32);
                }

                let mut line_names = track.line_names.into_iter();
                let mut values_iter = track.values.into_iter();
                let min_max_iter = ${self_grid}.mMinTrackSizingFunctions.iter_mut()
                                               .zip(${self_grid}.mMaxTrackSizingFunctions.iter_mut());

                for (i, (gecko_min, gecko_max)) in min_max_iter.enumerate().take(max_lines) {
                    let name_list = line_names.next().expect("expected line-names");
                    set_line_names(&name_list, &mut ${self_grid}.mLineNameLists[i]);
                    if i == auto_idx {
                        let track_size = auto_track_size.take().expect("expected <track-size> for <auto-track-repeat>");
                        track_size.to_gecko_style_coords(gecko_min, gecko_max);
                        continue
                    }

                    let track_size = values_iter.next().expect("expected <track-size> value");
                    track_size.to_gecko_style_coords(gecko_min, gecko_max);
                }

                let final_names = line_names.next().unwrap();
                set_line_names(&final_names, ${self_grid}.mLineNameLists.last_mut().unwrap());
            },
            GridTemplateComponent::None => {
                unsafe {
                    bindings::Gecko_SetStyleGridTemplateArrayLengths(&mut ${self_grid}, 0);
                    bindings::Gecko_ResizeTArrayForStrings(
                        &mut ${self_grid}.mRepeatAutoLineNameListBefore, 0);
                    bindings::Gecko_ResizeTArrayForStrings(
                        &mut ${self_grid}.mRepeatAutoLineNameListAfter, 0);
                }
            },
            GridTemplateComponent::Subgrid(mut list) => {
                ${self_grid}.set_mIsSubgrid(true);
                let names_length = match list.fill_idx {
                    Some(_) => list.names.len() - 1,
                    None => list.names.len(),
                };
                let num_values = cmp::min(names_length, max_lines + 1);

                unsafe {
                    bindings::Gecko_SetStyleGridTemplateArrayLengths(&mut ${self_grid}, 0);
                    bindings::Gecko_SetGridTemplateLineNamesLength(&mut ${self_grid}, num_values as u32);
                    bindings::Gecko_ResizeTArrayForStrings(
                        &mut ${self_grid}.mRepeatAutoLineNameListBefore, 0);
                    bindings::Gecko_ResizeTArrayForStrings(
                        &mut ${self_grid}.mRepeatAutoLineNameListAfter, 0);
                }

                if let Some(idx) = list.fill_idx {
                    ${self_grid}.set_mIsAutoFill(true);
                    ${self_grid}.mRepeatAutoIndex = idx as i16;
                    set_line_names(&list.names.swap_remove(idx as usize),
                                   &mut ${self_grid}.mRepeatAutoLineNameListBefore);
                }

                for (servo_names, gecko_names) in list.names.iter().zip(${self_grid}.mLineNameLists.iter_mut()) {
                    set_line_names(servo_names, gecko_names);
                }
            },
        }
    }

    pub fn copy_grid_template_${kind}_from(&mut self, other: &Self) {
        unsafe {
            bindings::Gecko_CopyStyleGridTemplateValues(&mut ${self_grid},
                                                        &other.gecko.mGridTemplate${kind.title()});
        }
    }
    % endfor

    ${impl_simple_type_with_conversion("grid_auto_flow")}

    pub fn set_grid_template_areas(&mut self, v: longhands::grid_template_areas::computed_value::T) {
        use gecko_bindings::bindings::Gecko_NewGridTemplateAreasValue;
        use gecko_bindings::sugar::refptr::UniqueRefPtr;

        let v = match v {
            Either::First(areas) => areas,
            Either::Second(_) => {
                unsafe { self.gecko.mGridTemplateAreas.clear() }
                return;
            },
        };

        let mut refptr = unsafe {
            UniqueRefPtr::from_addrefed(
                Gecko_NewGridTemplateAreasValue(v.areas.len() as u32, v.strings.len() as u32, v.width))
        };

        for (servo, gecko) in v.areas.into_iter().zip(refptr.mNamedAreas.iter_mut()) {
            gecko.mName.assign_utf8(&*servo.name);
            gecko.mColumnStart = servo.columns.start;
            gecko.mColumnEnd = servo.columns.end;
            gecko.mRowStart = servo.rows.start;
            gecko.mRowEnd = servo.rows.end;
        }

        for (servo, gecko) in v.strings.into_iter().zip(refptr.mTemplates.iter_mut()) {
            gecko.assign_utf8(&*servo);
        }

        unsafe { self.gecko.mGridTemplateAreas.set_move(refptr.get()) }
    }

    pub fn copy_grid_template_areas_from(&mut self, other: &Self) {
        unsafe { self.gecko.mGridTemplateAreas.set(&other.gecko.mGridTemplateAreas) }
    }
</%self:impl_trait>

<% skip_outline_longhands = " ".join("outline-style outline-width".split() +
                                     ["-moz-outline-radius-{0}".format(x.ident.replace("_", ""))
                                      for x in CORNERS]) %>
<%self:impl_trait style_struct_name="Outline"
                  skip_longhands="${skip_outline_longhands}"
                  skip_additionals="*">

    #[allow(non_snake_case)]
    pub fn set_outline_style(&mut self, v: longhands::outline_style::computed_value::T) {
        // FIXME(bholley): Align binary representations and ditch |match| for
        // cast + static_asserts
        let result = match v {
            % for value in border_style_keyword.values_for('gecko'):
                Either::Second(border_style::T::${to_rust_ident(value)}) =>
                    structs::${border_style_keyword.gecko_constant(value)} ${border_style_keyword.maybe_cast("u8")},
            % endfor
                Either::First(Auto) =>
                    structs::${border_style_keyword.gecko_constant('auto')} ${border_style_keyword.maybe_cast("u8")},
        };
        ${set_gecko_property("mOutlineStyle", "result")}

        // NB: This is needed to correctly handling the initial value of
        // outline-width when outline-style changes, see the
        // update_border_${side} comment for more details.
        self.gecko.mActualOutlineWidth = self.gecko.mOutlineWidth;
    }

    #[allow(non_snake_case)]
    pub fn copy_outline_style_from(&mut self, other: &Self) {
        self.gecko.mOutlineStyle = other.gecko.mOutlineStyle;
    }

    #[allow(non_snake_case)]
    pub fn clone_outline_style(&self) -> longhands::outline_style::computed_value::T {
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        match ${get_gecko_property("mOutlineStyle")} ${border_style_keyword.maybe_cast("u32")} {
            % for value in border_style_keyword.values_for('gecko'):
            structs::${border_style_keyword.gecko_constant(value)} => Either::Second(border_style::T::${value}),
            % endfor
            structs::${border_style_keyword.gecko_constant('auto')} => Either::First(Auto),
            % if border_style_keyword.gecko_inexhaustive:
            x => panic!("Found unexpected value in style struct for outline_style property: {:?}", x),
            % endif
        }
    }

    <% impl_app_units("outline_width", "mActualOutlineWidth",
                      inherit_from="mOutlineWidth",
                      need_clone=True, round_to_pixels=True) %>

    % for corner in CORNERS:
    <% impl_corner_style_coord("_moz_outline_radius_%s" % corner.ident.replace("_", ""),
                               "mOutlineRadius",
                               corner.x_index,
                               corner.y_index,
                               need_clone=True) %>
    % endfor

    pub fn outline_has_nonzero_width(&self) -> bool {
        self.gecko.mActualOutlineWidth != 0
    }
</%self:impl_trait>

<%
    skip_font_longhands = """font-family font-size font-size-adjust font-weight
                             font-synthesis -x-lang font-variant-alternates
                             font-variant-east-asian font-variant-ligatures
                             font-variant-numeric font-language-override
                             font-feature-settings font-variation-settings
                             -moz-min-font-size-ratio"""
%>
<%self:impl_trait style_struct_name="Font"
    skip_longhands="${skip_font_longhands}"
    skip_additionals="*">

    pub fn set_font_feature_settings(&mut self, v: longhands::font_feature_settings::computed_value::T) {
        use values::generics::FontSettings;

        let current_settings = &mut self.gecko.mFont.fontFeatureSettings;
        current_settings.clear_pod();

        match v {
            FontSettings::Normal => (), // do nothing, length is already 0

            FontSettings::Tag(feature_settings) => {
                unsafe { current_settings.set_len_pod(feature_settings.len() as u32) };

                for (current, feature) in current_settings.iter_mut().zip(feature_settings) {
                    current.mTag = feature.tag;
                    current.mValue = feature.value.0;
                }
            }
        };
    }

    pub fn copy_font_feature_settings_from(&mut self, other: &Self ) {
        let current_settings = &mut self.gecko.mFont.fontFeatureSettings;
        let feature_settings = &other.gecko.mFont.fontFeatureSettings;
        let settings_length = feature_settings.len() as u32;

        current_settings.clear_pod();
        unsafe { current_settings.set_len_pod(settings_length) };

        for (current, feature) in current_settings.iter_mut().zip(feature_settings.iter()) {
            current.mTag = feature.mTag;
            current.mValue = feature.mValue;
        }
    }

    pub fn clone_font_feature_settings(&self) -> longhands::font_feature_settings::computed_value::T {
        use values::generics::{FontSettings, FontSettingTag, FontSettingTagInt} ;

        if self.gecko.mFont.fontFeatureSettings.len() == 0 {
            FontSettings::Normal
        } else {
            FontSettings::Tag(
                self.gecko.mFont.fontFeatureSettings.iter().map(|gecko_font_feature_setting| {
                    FontSettingTag {
                        tag: gecko_font_feature_setting.mTag,
                        value: FontSettingTagInt(gecko_font_feature_setting.mValue),
                    }
                }).collect()
            )
        }
    }

    pub fn set_font_variation_settings(&mut self, v: longhands::font_variation_settings::computed_value::T) {
        use values::generics::FontSettings;

        let current_settings = &mut self.gecko.mFont.fontVariationSettings;
        current_settings.clear_pod();

        match v {
            FontSettings::Normal => (), // do nothing, length is already 0

            FontSettings::Tag(feature_settings) => {
                unsafe { current_settings.set_len_pod(feature_settings.len() as u32) };

                for (current, feature) in current_settings.iter_mut().zip(feature_settings) {
                    current.mTag = feature.tag;
                    current.mValue = feature.value.0;
                }
            }
        };
    }

    pub fn copy_font_variation_settings_from(&mut self, other: &Self ) {
        let current_settings = &mut self.gecko.mFont.fontVariationSettings;
        let feature_settings = &other.gecko.mFont.fontVariationSettings;
        let settings_length = feature_settings.len() as u32;

        current_settings.clear_pod();
        unsafe { current_settings.set_len_pod(settings_length) };

        for (current, feature) in current_settings.iter_mut().zip(feature_settings.iter()) {
            current.mTag = feature.mTag;
            current.mValue = feature.mValue;
        }
    }

    pub fn fixup_none_generic(&mut self, device: &Device) {
        unsafe {
            bindings::Gecko_nsStyleFont_FixupNoneGeneric(&mut self.gecko, device.pres_context())
        }
    }

    pub fn set_font_family(&mut self, v: longhands::font_family::computed_value::T) {
        use properties::longhands::font_family::computed_value::FontFamily;

        let list = &mut self.gecko.mFont.fontlist;
        unsafe { Gecko_FontFamilyList_Clear(list); }

        self.gecko.mGenericID = structs::kGenericFont_NONE;

        for family in &v.0 {
            match *family {
                FontFamily::FamilyName(ref f) => {
                    unsafe { Gecko_FontFamilyList_AppendNamed(list, f.name.as_ptr(), f.quoted); }
                }
                FontFamily::Generic(ref name) => {
                    let (family_type, generic) = FontFamily::generic(name);
                    if v.0.len() == 1 {
                        self.gecko.mGenericID = generic;
                    }
                    unsafe { Gecko_FontFamilyList_AppendGeneric(list, family_type); }
                }
            }
        }
    }

    pub fn font_family_count(&self) -> usize {
        0
    }

    pub fn font_family_at(&self, _: usize) -> longhands::font_family::computed_value::FontFamily {
        unimplemented!()
    }

    pub fn copy_font_family_from(&mut self, other: &Self) {
        unsafe { Gecko_CopyFontFamilyFrom(&mut self.gecko.mFont, &other.gecko.mFont); }
        self.gecko.mGenericID = other.gecko.mGenericID;
    }

    pub fn clone_font_family(&self) -> longhands::font_family::computed_value::T {
        use properties::longhands::font_family::computed_value::{FontFamily, FamilyName};
        use gecko_bindings::structs::FontFamilyType;
        use gecko_string_cache::Atom;

        ::properties::longhands::font_family::computed_value::T(
            self.gecko.mFont.fontlist.mFontlist.iter().map(|gecko_font_family_name| {
                match gecko_font_family_name.mType {
                    FontFamilyType::eFamily_serif => FontFamily::Generic(atom!("serif")),
                    FontFamilyType::eFamily_sans_serif => FontFamily::Generic(atom!("sans-serif")),
                    FontFamilyType::eFamily_monospace => FontFamily::Generic(atom!("monospace")),
                    FontFamilyType::eFamily_cursive => FontFamily::Generic(atom!("cursive")),
                    FontFamilyType::eFamily_fantasy => FontFamily::Generic(atom!("fantasy")),
                    FontFamilyType::eFamily_moz_fixed => FontFamily::Generic(Atom::from("-moz-fixed")),
                    FontFamilyType::eFamily_named => FontFamily::FamilyName(FamilyName {
                        name: (&*gecko_font_family_name.mName).into(),
                        quoted: false
                    }),
                    FontFamilyType::eFamily_named_quoted => FontFamily::FamilyName(FamilyName {
                        name: (&*gecko_font_family_name.mName).into(),
                        quoted: true
                    }),
                    x => panic!("Found unexpected font FontFamilyType: {:?}", x),
                }
            }).collect()
        )
    }

    // FIXME(bholley): Gecko has two different sizes, one of which (mSize) is the
    // actual computed size, and the other of which (mFont.size) is the 'display
    // size' which takes font zooming into account. We don't handle font zooming yet.
    pub fn set_font_size(&mut self, v: longhands::font_size::computed_value::T) {
        self.gecko.mSize = v.0;
        self.gecko.mScriptUnconstrainedSize = v.0;
    }

    /// Set font size, taking into account scriptminsize and scriptlevel
    /// Returns Some(size) if we have to recompute the script unconstrained size
    pub fn apply_font_size(&mut self, v: longhands::font_size::computed_value::T,
                           parent: &Self,
                           device: &Device) -> Option<Au> {
        let (adjusted_size, adjusted_unconstrained_size)
            = self.calculate_script_level_size(parent);
        // In this case, we have been unaffected by scriptminsize, ignore it
        if parent.gecko.mSize == parent.gecko.mScriptUnconstrainedSize &&
           adjusted_size == adjusted_unconstrained_size {
            self.set_font_size(v);
            self.fixup_font_min_size(device);
            None
        } else {
            self.gecko.mSize = v.0;
            self.fixup_font_min_size(device);
            Some(Au(parent.gecko.mScriptUnconstrainedSize))
        }
    }

    pub fn fixup_font_min_size(&mut self, device: &Device) {
        unsafe { bindings::Gecko_nsStyleFont_FixupMinFontSize(&mut self.gecko, device.pres_context()) }
    }

    pub fn apply_unconstrained_font_size(&mut self, v: Au) {
        self.gecko.mScriptUnconstrainedSize = v.0;
    }

    /// Calculates the constrained and unconstrained font sizes to be inherited
    /// from the parent.
    ///
    /// See ComputeScriptLevelSize in Gecko's nsRuleNode.cpp
    ///
    /// scriptlevel is a property that affects how font-size is inherited. If scriptlevel is
    /// +1, for example, it will inherit as the script size multiplier times
    /// the parent font. This does not affect cases where the font-size is
    /// explicitly set.
    ///
    /// However, this transformation is not allowed to reduce the size below
    /// scriptminsize. If this inheritance will reduce it to below
    /// scriptminsize, it will be set to scriptminsize or the parent size,
    /// whichever is smaller (the parent size could be smaller than the min size
    /// because it was explicitly specified).
    ///
    /// Now, within a node that has inherited a font-size which was
    /// crossing scriptminsize once the scriptlevel was applied, a negative
    /// scriptlevel may be used to increase the size again.
    ///
    /// This should work, however if we have already been capped by the
    /// scriptminsize multiple times, this can lead to a jump in the size.
    ///
    /// For example, if we have text of the form:
    ///
    /// huge large medium small tiny reallytiny tiny small medium huge
    ///
    /// which is represented by progressive nesting and scriptlevel values of
    /// +1 till the center after which the scriptlevel is -1, the "tiny"s should
    /// be the same size, as should be the "small"s and "medium"s, etc.
    ///
    /// However, if scriptminsize kicked it at around "medium", then
    /// medium/tiny/reallytiny will all be the same size (the min size).
    /// A -1 scriptlevel change after this will increase the min size by the
    /// multiplier, making the second tiny larger than medium.
    ///
    /// Instead, we wish for the second "tiny" to still be capped by the script
    /// level, and when we reach the second "large", it should be the same size
    /// as the original one.
    ///
    /// We do this by cascading two separate font sizes. The font size (mSize)
    /// is the actual displayed font size. The unconstrained font size
    /// (mScriptUnconstrainedSize) is the font size in the situation where
    /// scriptminsize never applied.
    ///
    /// We calculate the proposed inherited font size based on scriptlevel and
    /// the parent unconstrained size, instead of using the parent font size.
    /// This is stored in the node's unconstrained size and will also be stored
    /// in the font size provided that it is above the min size.
    ///
    /// All of this only applies when inheriting. When the font size is
    /// manually set, scriptminsize does not apply, and both the real and
    /// unconstrained size are set to the explicit value. However, if the font
    /// size is manually set to an em or percent unit, the unconstrained size
    /// will be set to the value of that unit computed against the parent
    /// unconstrained size, whereas the font size will be set computing against
    /// the parent font size.
    pub fn calculate_script_level_size(&self, parent: &Self) -> (Au, Au) {
        use std::cmp;

        let delta = self.gecko.mScriptLevel - parent.gecko.mScriptLevel;

        let parent_size = Au(parent.gecko.mSize);
        let parent_unconstrained_size = Au(parent.gecko.mScriptUnconstrainedSize);

        if delta == 0 {
            return (parent_size, parent_unconstrained_size)
        }

        /// XXXManishearth this should also handle text zoom
        let min = Au(parent.gecko.mScriptMinSize);

        let scale = (parent.gecko.mScriptSizeMultiplier as f32).powi(delta as i32);

        let new_size = parent_size.scale_by(scale);
        let new_unconstrained_size = parent_unconstrained_size.scale_by(scale);

        if scale < 1. {
            // The parent size can be smaller than scriptminsize,
            // e.g. if it was specified explicitly. Don't scale
            // in this case, but we don't want to set it to scriptminsize
            // either since that will make it larger.
            if parent_size < min {
                (parent_size, new_unconstrained_size)
            } else {
                (cmp::max(min, new_size), new_unconstrained_size)
            }
        } else {
            // If the new unconstrained size is larger than the min size,
            // this means we have escaped the grasp of scriptminsize
            // and can revert to using the unconstrained size.
            // However, if the new size is even larger (perhaps due to usage
            // of em units), use that instead.
            (cmp::min(new_size, cmp::max(new_unconstrained_size, min)),
             new_unconstrained_size)
        }
    }

    /// This function will also handle scriptminsize and scriptlevel
    /// so should not be called when you just want the font sizes to be copied.
    /// Hence the different name.
    ///
    /// Returns true if the inherited keyword size was actually used
    pub fn inherit_font_size_from(&mut self, parent: &Self,
                                  kw_inherited_size: Option<Au>,
                                  device: &Device) -> bool {
        let (adjusted_size, adjusted_unconstrained_size)
            = self.calculate_script_level_size(parent);
        if adjusted_size.0 != parent.gecko.mSize ||
           adjusted_unconstrained_size.0 != parent.gecko.mScriptUnconstrainedSize {
            // This is incorrect. When there is both a keyword size being inherited
            // and a scriptlevel change, we must handle the keyword size the same
            // way we handle em units. This complicates things because we now have
            // to keep track of the adjusted and unadjusted ratios in the kw font size.
            // This only affects the use case of a generic font being used in MathML.
            //
            // If we were to fix this I would prefer doing it by removing the
            // ruletree walk on the Gecko side in nsRuleNode::SetGenericFont
            // and instead using extra bookkeeping in the mSize and mScriptUnconstrainedSize
            // values, and reusing those instead of font_size_keyword.


            // In the case that MathML has given us an adjusted size, apply it.
            // Keep track of the unconstrained adjusted size.
            self.gecko.mSize = adjusted_size.0;
            self.gecko.mScriptUnconstrainedSize = adjusted_unconstrained_size.0;
            self.fixup_font_min_size(device);
            false
        } else if let Some(size) = kw_inherited_size {
            // Parent element was a keyword-derived size.
            self.gecko.mSize = size.0;
            // MathML constraints didn't apply here, so we can ignore this.
            self.gecko.mScriptUnconstrainedSize = size.0;
            self.fixup_font_min_size(device);
            true
        } else {
            // MathML isn't affecting us, and our parent element does not
            // have a keyword-derived size. Set things normally.
            self.gecko.mSize = parent.gecko.mSize;
            self.gecko.mScriptUnconstrainedSize = parent.gecko.mScriptUnconstrainedSize;
            self.fixup_font_min_size(device);
            false
        }
    }

    pub fn clone_font_size(&self) -> longhands::font_size::computed_value::T {
        Au(self.gecko.mSize)
    }

    pub fn set_font_weight(&mut self, v: longhands::font_weight::computed_value::T) {
        self.gecko.mFont.weight = v.0;
    }
    ${impl_simple_copy('font_weight', 'mFont.weight')}

    pub fn clone_font_weight(&self) -> longhands::font_weight::computed_value::T {
        debug_assert!(self.gecko.mFont.weight >= 100);
        debug_assert!(self.gecko.mFont.weight <= 900);
        debug_assert!(self.gecko.mFont.weight % 10 == 0);
        unsafe { transmute(self.gecko.mFont.weight) }
    }

    ${impl_simple_type_with_conversion("font_synthesis", "mFont.synthesis")}

    pub fn set_font_size_adjust(&mut self, v: longhands::font_size_adjust::computed_value::T) {
        use properties::longhands::font_size_adjust::computed_value::T;
        match v {
            T::None => self.gecko.mFont.sizeAdjust = -1.0 as f32,
            T::Number(n) => self.gecko.mFont.sizeAdjust = n,
        }
    }

    pub fn copy_font_size_adjust_from(&mut self, other: &Self) {
        self.gecko.mFont.sizeAdjust = other.gecko.mFont.sizeAdjust;
    }

    pub fn clone_font_size_adjust(&self) -> longhands::font_size_adjust::computed_value::T {
        use properties::longhands::font_size_adjust::computed_value::T;
        T::from_gecko_adjust(self.gecko.mFont.sizeAdjust)
    }

    #[allow(non_snake_case)]
    pub fn set__x_lang(&mut self, v: longhands::_x_lang::computed_value::T) {
        let ptr = v.0.as_ptr();
        forget(v);
        unsafe {
            Gecko_nsStyleFont_SetLang(&mut self.gecko, ptr);
        }
    }

    #[allow(non_snake_case)]
    pub fn copy__x_lang_from(&mut self, other: &Self) {
        unsafe {
            Gecko_nsStyleFont_CopyLangFrom(&mut self.gecko, &other.gecko);
        }
    }

    <% impl_simple_type_with_conversion("font_language_override", "mFont.languageOverride") %>

    pub fn set_font_variant_alternates(&mut self, v: longhands::font_variant_alternates::computed_value::T) {
        use gecko_bindings::bindings::{Gecko_ClearAlternateValues, Gecko_AppendAlternateValues};
        % for value in "normal swash stylistic ornaments annotation styleset character_variant historical".split():
            use gecko_bindings::structs::NS_FONT_VARIANT_ALTERNATES_${value.upper()};
        % endfor
        use self::longhands::font_variant_alternates::VariantAlternates;

        unsafe {
            Gecko_ClearAlternateValues(&mut self.gecko.mFont, v.len());
        }

        if v.0.is_empty() {
            self.gecko.mFont.variantAlternates = NS_FONT_VARIANT_ALTERNATES_NORMAL as u16;
        }

        for val in v.0.iter() {
            match *val {
                % for value in "Swash Stylistic Ornaments Annotation".split():
                    VariantAlternates::${value}(ref ident) => {
                        self.gecko.mFont.variantAlternates |= NS_FONT_VARIANT_ALTERNATES_${value.upper()} as u16;
                        unsafe {
                            Gecko_AppendAlternateValues(&mut self.gecko.mFont,
                                                        NS_FONT_VARIANT_ALTERNATES_${value.upper()},
                                                        ident.0.as_ptr());
                        }
                    },
                % endfor
                % for value in "styleset character_variant".split():
                    VariantAlternates::${to_camel_case(value)}(ref slice) => {
                        self.gecko.mFont.variantAlternates |= NS_FONT_VARIANT_ALTERNATES_${value.upper()} as u16;
                        for ident in slice.iter() {
                            unsafe {
                                Gecko_AppendAlternateValues(&mut self.gecko.mFont,
                                                            NS_FONT_VARIANT_ALTERNATES_${value.upper()},
                                                            ident.0.as_ptr());
                            }
                        }
                    },
                % endfor
                VariantAlternates::HistoricalForms => {
                    self.gecko.mFont.variantAlternates |= NS_FONT_VARIANT_ALTERNATES_HISTORICAL as u16;
                }
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn copy_font_variant_alternates_from(&mut self, other: &Self) {
        use gecko_bindings::bindings::Gecko_CopyAlternateValuesFrom;

        self.gecko.mFont.variantAlternates = other.gecko.mFont.variantAlternates;
        unsafe {
            Gecko_CopyAlternateValuesFrom(&mut self.gecko.mFont, &other.gecko.mFont);
        }
    }

    ${impl_simple_type_with_conversion("font_variant_ligatures", "mFont.variantLigatures")}
    ${impl_simple_type_with_conversion("font_variant_east_asian", "mFont.variantEastAsian")}
    ${impl_simple_type_with_conversion("font_variant_numeric", "mFont.variantNumeric")}

    #[allow(non_snake_case)]
    pub fn set__moz_min_font_size_ratio(&mut self, v: longhands::_moz_min_font_size_ratio::computed_value::T) {
        let percentage = if v.0 > 255. {
            255.
        } else if v.0 < 0. {
            0.
        } else {
            v.0
        };

        self.gecko.mMinFontSizeRatio = percentage as u8;
    }

    ${impl_simple_copy('_moz_min_font_size_ratio', 'mMinFontSizeRatio')}
</%self:impl_trait>

<%def name="impl_copy_animation_or_transition_value(type, ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn copy_${type}_${ident}_from(&mut self, other: &Self) {
        unsafe { self.gecko.m${type.capitalize()}s.ensure_len(other.gecko.m${type.capitalize()}s.len()) };

        let count = other.gecko.m${type.capitalize()}${gecko_ffi_name}Count;
        self.gecko.m${type.capitalize()}${gecko_ffi_name}Count = count;

        // The length of mTransitions or mAnimations is often greater than m{Transition|Animation}XXCount,
        // don't copy values over the count.
        for (index, gecko) in self.gecko.m${type.capitalize()}s.iter_mut().enumerate().take(count as usize) {
            gecko.m${gecko_ffi_name} = other.gecko.m${type.capitalize()}s[index].m${gecko_ffi_name};
        }
    }
</%def>

<%def name="impl_animation_or_transition_count(type, ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn ${type}_${ident}_count(&self) -> usize {
        self.gecko.m${type.capitalize()}${gecko_ffi_name}Count as usize
    }
</%def>

<%def name="impl_animation_or_transition_time_value(type, ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn set_${type}_${ident}<I>(&mut self, v: I)
        where I: IntoIterator<Item = longhands::${type}_${ident}::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator + Clone
    {
        let v = v.into_iter();
        debug_assert!(v.len() != 0);
        let input_len = v.len();
        unsafe { self.gecko.m${type.capitalize()}s.ensure_len(input_len) };

        self.gecko.m${type.capitalize()}${gecko_ffi_name}Count = input_len as u32;
        for (gecko, servo) in self.gecko.m${type.capitalize()}s.iter_mut().zip(v.cycle()) {
            gecko.m${gecko_ffi_name} = servo.seconds() * 1000.;
        }
    }
    #[allow(non_snake_case)]
    pub fn ${type}_${ident}_at(&self, index: usize)
        -> longhands::${type}_${ident}::computed_value::SingleComputedValue {
        use values::computed::Time;
        Time::from_seconds(self.gecko.m${type.capitalize()}s[index].m${gecko_ffi_name} / 1000.)
    }
    ${impl_animation_or_transition_count(type, ident, gecko_ffi_name)}
    ${impl_copy_animation_or_transition_value(type, ident, gecko_ffi_name)}
</%def>

<%def name="impl_animation_or_transition_timing_function(type)">
    pub fn set_${type}_timing_function<I>(&mut self, v: I)
        where I: IntoIterator<Item = longhands::${type}_timing_function::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator + Clone
    {
        let v = v.into_iter();
        debug_assert!(v.len() != 0);
        let input_len = v.len();
        unsafe { self.gecko.m${type.capitalize()}s.ensure_len(input_len) };

        self.gecko.m${type.capitalize()}TimingFunctionCount = input_len as u32;
        for (gecko, servo) in self.gecko.m${type.capitalize()}s.iter_mut().zip(v.cycle()) {
            gecko.mTimingFunction = servo.into();
        }
    }
    ${impl_animation_or_transition_count(type, 'timing_function', 'TimingFunction')}
    ${impl_copy_animation_or_transition_value(type, 'timing_function', 'TimingFunction')}
    pub fn ${type}_timing_function_at(&self, index: usize)
        -> longhands::${type}_timing_function::computed_value::SingleComputedValue {
        self.gecko.m${type.capitalize()}s[index].mTimingFunction.into()
    }
</%def>

<%def name="impl_transition_time_value(ident, gecko_ffi_name)">
    ${impl_animation_or_transition_time_value('transition', ident, gecko_ffi_name)}
</%def>

<%def name="impl_transition_count(ident, gecko_ffi_name)">
    ${impl_animation_or_transition_count('transition', ident, gecko_ffi_name)}
</%def>

<%def name="impl_copy_animation_value(ident, gecko_ffi_name)">
    ${impl_copy_animation_or_transition_value('animation', ident, gecko_ffi_name)}
</%def>

<%def name="impl_transition_timing_function()">
    ${impl_animation_or_transition_timing_function('transition')}
</%def>

<%def name="impl_animation_count(ident, gecko_ffi_name)">
    ${impl_animation_or_transition_count('animation', ident, gecko_ffi_name)}
</%def>

<%def name="impl_animation_time_value(ident, gecko_ffi_name)">
    ${impl_animation_or_transition_time_value('animation', ident, gecko_ffi_name)}
</%def>

<%def name="impl_animation_timing_function()">
    ${impl_animation_or_transition_timing_function('animation')}
</%def>

<%def name="impl_animation_keyword(ident, gecko_ffi_name, keyword, cast_type='u8')">
    #[allow(non_snake_case)]
    pub fn set_animation_${ident}<I>(&mut self, v: I)
        where I: IntoIterator<Item = longhands::animation_${ident}::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator + Clone
    {
        use properties::longhands::animation_${ident}::single_value::computed_value::T as Keyword;
        use gecko_bindings::structs;

        let v = v.into_iter();

        debug_assert!(v.len() != 0);
        let input_len = v.len();
        unsafe { self.gecko.mAnimations.ensure_len(input_len) };

        self.gecko.mAnimation${gecko_ffi_name}Count = input_len as u32;

        for (gecko, servo) in self.gecko.mAnimations.iter_mut().zip(v.cycle()) {
            let result = match servo {
                % for value in keyword.gecko_values():
                    Keyword::${to_rust_ident(value)} =>
                        structs::${keyword.gecko_constant(value)} ${keyword.maybe_cast(cast_type)},
                % endfor
            };
            gecko.m${gecko_ffi_name} = result;
        }
    }
    #[allow(non_snake_case)]
    pub fn animation_${ident}_at(&self, index: usize)
        -> longhands::animation_${ident}::computed_value::SingleComputedValue {
        use properties::longhands::animation_${ident}::single_value::computed_value::T as Keyword;
        match self.gecko.mAnimations[index].m${gecko_ffi_name} ${keyword.maybe_cast("u32")} {
            % for value in keyword.gecko_values():
                structs::${keyword.gecko_constant(value)} => Keyword::${to_rust_ident(value)},
            % endfor
            x => panic!("Found unexpected value for animation-${ident}: {:?}", x),
        }
    }
    ${impl_animation_count(ident, gecko_ffi_name)}
    ${impl_copy_animation_value(ident, gecko_ffi_name)}
</%def>

<% skip_box_longhands= """display overflow-y vertical-align
                          animation-name animation-delay animation-duration
                          animation-direction animation-fill-mode animation-play-state
                          animation-iteration-count animation-timing-function
                          transition-duration transition-delay
                          transition-timing-function transition-property
                          page-break-before page-break-after
                          scroll-snap-points-x scroll-snap-points-y transform
                          scroll-snap-type-y scroll-snap-coordinate
                          perspective-origin transform-origin -moz-binding will-change
                          shape-outside contain touch-action""" %>
<%self:impl_trait style_struct_name="Box" skip_longhands="${skip_box_longhands}">

    // We manually-implement the |display| property until we get general
    // infrastructure for preffing certain values.
    <% display_keyword = Keyword("display", "inline block inline-block table inline-table table-row-group " +
                                            "table-header-group table-footer-group table-row table-column-group " +
                                            "table-column table-cell table-caption list-item flex none " +
                                            "inline-flex grid inline-grid ruby ruby-base ruby-base-container " +
                                            "ruby-text ruby-text-container contents flow-root -webkit-box " +
                                            "-webkit-inline-box -moz-box -moz-inline-box -moz-grid -moz-inline-grid " +
                                            "-moz-grid-group -moz-grid-line -moz-stack -moz-inline-stack -moz-deck " +
                                            "-moz-popup -moz-groupbox",
                                            gecko_enum_prefix="StyleDisplay",
                                            gecko_inexhaustive="True",
                                            gecko_strip_moz_prefix=False) %>

    pub fn set_display(&mut self, v: longhands::display::computed_value::T) {
        use properties::longhands::display::computed_value::T as Keyword;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        let result = match v {
            % for value in display_keyword.values_for('gecko'):
                Keyword::${to_rust_ident(value)} =>
                    structs::${display_keyword.gecko_constant(value)},
            % endfor
        };
        self.gecko.mDisplay = result;
        self.gecko.mOriginalDisplay = result;
    }

    /// Set the display value from the style adjustment code. This is pretty
    /// much like set_display, but without touching the mOriginalDisplay field,
    /// which we want to keep.
    pub fn set_adjusted_display(&mut self,
                                v: longhands::display::computed_value::T,
                                _is_item_or_root: bool) {
        use properties::longhands::display::computed_value::T as Keyword;
        let result = match v {
            % for value in display_keyword.values_for('gecko'):
                Keyword::${to_rust_ident(value)} =>
                    structs::${display_keyword.gecko_constant(value)},
            % endfor
        };
        self.gecko.mDisplay = result;
    }

    pub fn copy_display_from(&mut self, other: &Self) {
        self.gecko.mDisplay = other.gecko.mDisplay;
        self.gecko.mOriginalDisplay = other.gecko.mDisplay;
    }

    <%call expr="impl_keyword_clone('display', 'mDisplay', display_keyword)"></%call>

    <% overflow_x = data.longhands_by_name["overflow-x"] %>
    pub fn set_overflow_y(&mut self, v: longhands::overflow_y::computed_value::T) {
        use properties::longhands::overflow_x::computed_value::T as BaseType;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        self.gecko.mOverflowY = match v {
            % for value in overflow_x.keyword.values_for('gecko'):
                BaseType::${to_rust_ident(value)} => structs::${overflow_x.keyword.gecko_constant(value)} as u8,
            % endfor
        };
    }
    ${impl_simple_copy('overflow_y', 'mOverflowY')}
    pub fn clone_overflow_y(&self) -> longhands::overflow_y::computed_value::T {
        use properties::longhands::overflow_x::computed_value::T as BaseType;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        match self.gecko.mOverflowY as u32 {
            % for value in overflow_x.keyword.values_for('gecko'):
            structs::${overflow_x.keyword.gecko_constant(value)} => BaseType::${to_rust_ident(value)},
            % endfor
            x => panic!("Found unexpected value in style struct for overflow_y property: {}", x),
        }
    }

    pub fn set_vertical_align(&mut self, v: longhands::vertical_align::computed_value::T) {
        <% keyword = data.longhands_by_name["vertical-align"].keyword %>
        use properties::longhands::vertical_align::computed_value::T;
        // FIXME: Align binary representations and ditch |match| for cast + static_asserts
        match v {
            % for value in keyword.values_for('gecko'):
                T::${to_rust_ident(value)} =>
                    self.gecko.mVerticalAlign.set_value(
                            CoordDataValue::Enumerated(structs::${keyword.gecko_constant(value)})),
            % endfor
            T::LengthOrPercentage(v) => self.gecko.mVerticalAlign.set(v),
        }
    }

    pub fn clone_vertical_align(&self) -> longhands::vertical_align::computed_value::T {
        use properties::longhands::vertical_align::computed_value::T;
        use values::computed::LengthOrPercentage;

        match self.gecko.mVerticalAlign.as_value() {
            % for value in keyword.values_for('gecko'):
                CoordDataValue::Enumerated(structs::${keyword.gecko_constant(value)}) => T::${to_rust_ident(value)},
            % endfor
                CoordDataValue::Enumerated(_) => panic!("Unexpected enum variant for vertical-align"),
                _ => {
                    let v = LengthOrPercentage::from_gecko_style_coord(&self.gecko.mVerticalAlign)
                        .expect("Expected length or percentage for vertical-align");
                    T::LengthOrPercentage(v)
                }
        }
    }

    <%call expr="impl_coord_copy('vertical_align', 'mVerticalAlign')"></%call>

    % for kind in ["before", "after"]:
    // Temp fix for Bugzilla bug 24000.
    // Map 'auto' and 'avoid' to false, and 'always', 'left', and 'right' to true.
    // "A conforming user agent may interpret the values 'left' and 'right'
    // as 'always'." - CSS2.1, section 13.3.1
    pub fn set_page_break_${kind}(&mut self, v: longhands::page_break_${kind}::computed_value::T) {
        use computed_values::page_break_${kind}::T;

        let result = match v {
            T::auto   => false,
            T::always => true,
            T::avoid  => false,
            T::left   => true,
            T::right  => true
        };
        self.gecko.mBreak${kind.title()} = result;
    }

    ${impl_simple_copy('page_break_' + kind, 'mBreak' + kind.title())}

    // Temp fix for Bugzilla bug 24000.
    // See set_page_break_before/after for detail.
    pub fn clone_page_break_${kind}(&self) -> longhands::page_break_${kind}::computed_value::T {
        use computed_values::page_break_${kind}::T;

        match self.gecko.mBreak${kind.title()} {
            true => T::always,
            false => T::auto,
        }
    }
    % endfor

    ${impl_style_coord("scroll_snap_points_x", "mScrollSnapPointsX", True)}
    ${impl_style_coord("scroll_snap_points_y", "mScrollSnapPointsY", True)}

    pub fn set_scroll_snap_coordinate<I>(&mut self, v: I)
        where I: IntoIterator<Item = longhands::scroll_snap_coordinate::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        let v = v.into_iter();

        unsafe { self.gecko.mScrollSnapCoordinate.set_len_pod(v.len() as u32); }
        for (gecko, servo) in self.gecko.mScrollSnapCoordinate
                               .iter_mut()
                               .zip(v) {
            gecko.mXPosition = servo.horizontal.into();
            gecko.mYPosition = servo.vertical.into();
        }
    }

    pub fn copy_scroll_snap_coordinate_from(&mut self, other: &Self) {
        unsafe {
            self.gecko.mScrollSnapCoordinate
                .set_len_pod(other.gecko.mScrollSnapCoordinate.len() as u32);
        }

        for (this, that) in self.gecko.mScrollSnapCoordinate
                               .iter_mut()
                               .zip(other.gecko.mScrollSnapCoordinate.iter()) {
            *this = *that;
        }
    }

    pub fn clone_scroll_snap_coordinate(&self) -> longhands::scroll_snap_coordinate::computed_value::T {
        let vec = self.gecko.mScrollSnapCoordinate.iter().map(|f| f.into()).collect();
        longhands::scroll_snap_coordinate::computed_value::T(vec)
    }

    ${impl_css_url('_moz_binding', 'mBinding.mPtr')}

    <%def name="transform_function_arm(name, keyword, items)">
        <%
            pattern = None
            if keyword == "matrix3d":
                # m11: number1, m12: number2, ..
                single_patterns = ["m%s: %s" % (str(a / 4 + 1) + str(a % 4 + 1), b + str(a + 1)) for (a, b)
                                   in enumerate(items)]
                if name == "Matrix":
                    pattern = "(ComputedMatrix { %s })" % ", ".join(single_patterns)
                else:
                    pattern = "(ComputedMatrixWithPercents { %s })" % ", ".join(single_patterns)
            elif keyword == "interpolatematrix":
                pattern = " { from_list: ref list1, to_list: ref list2, progress: percentage3 }"
            elif keyword == "accumulatematrix":
                pattern = " { from_list: ref list1, to_list: ref list2, count: integer_to_percentage3 }"
            else:
                # Generate contents of pattern from items
                pattern = "(%s)" % ", ".join([b + str(a+1) for (a,b) in enumerate(items)])

            # First %s substituted with the call to GetArrayItem, the second
            # %s substituted with the corresponding variable
            css_value_setters = {
                "length" : "bindings::Gecko_CSSValue_SetAbsoluteLength(%s, %s.0)",
                "percentage" : "bindings::Gecko_CSSValue_SetPercentage(%s, %s.0)",
                # Note: This is an integer type, but we use it as a percentage value in Gecko, so
                #       need to cast it to f32.
                "integer_to_percentage" : "bindings::Gecko_CSSValue_SetPercentage(%s, %s as f32)",
                "lop" : "%s.set_lop(%s)",
                "angle" : "%s.set_angle(%s)",
                "number" : "bindings::Gecko_CSSValue_SetNumber(%s, %s)",
                # Note: We use nsCSSValueSharedList here, instead of nsCSSValueList_heap
                #       because this function is not called on the main thread and
                #       nsCSSValueList_heap is not thread safe.
                "list" : "%s.set_shared_list(%s.0.as_ref().unwrap().into_iter().map(&convert_to_ns_css_value));",
            }
        %>
        longhands::transform::computed_value::ComputedOperation::${name}${pattern} => {
            bindings::Gecko_CSSValue_SetFunction(gecko_value, ${len(items) + 1});
            bindings::Gecko_CSSValue_SetKeyword(
                bindings::Gecko_CSSValue_GetArrayItem(gecko_value, 0),
                structs::nsCSSKeyword::eCSSKeyword_${keyword}
            );
            % for index, item in enumerate(items):
                % if item == "list":
                    debug_assert!(${item}${index + 1}.0.is_some());
                % endif
                ${css_value_setters[item] % (
                    "bindings::Gecko_CSSValue_GetArrayItem(gecko_value, %d)" % (index + 1),
                    item + str(index + 1)
                )};
            % endfor
        }
    </%def>
    fn set_single_transform_function(servo_value: &longhands::transform::computed_value::ComputedOperation,
                                     gecko_value: &mut structs::nsCSSValue /* output */) {
        use properties::longhands::transform::computed_value::ComputedMatrix;
        use properties::longhands::transform::computed_value::ComputedMatrixWithPercents;
        use properties::longhands::transform::computed_value::ComputedOperation;

        let convert_to_ns_css_value = |item: &ComputedOperation| -> structs::nsCSSValue {
            let mut value = structs::nsCSSValue::null();
            Self::set_single_transform_function(item, &mut value);
            value
        };

        unsafe {
            match *servo_value {
                ${transform_function_arm("Matrix", "matrix3d", ["number"] * 16)}
                ${transform_function_arm("MatrixWithPercents", "matrix3d", ["number"] * 12 + ["lop"] * 2
                                         + ["length"] + ["number"])}
                ${transform_function_arm("Skew", "skew", ["angle"] * 2)}
                ${transform_function_arm("Translate", "translate3d", ["lop", "lop", "length"])}
                ${transform_function_arm("Scale", "scale3d", ["number"] * 3)}
                ${transform_function_arm("Rotate", "rotate3d", ["number"] * 3 + ["angle"])}
                ${transform_function_arm("Perspective", "perspective", ["length"])}
                ${transform_function_arm("InterpolateMatrix", "interpolatematrix",
                                         ["list"] * 2 + ["percentage"])}
                ${transform_function_arm("AccumulateMatrix", "accumulatematrix",
                                         ["list"] * 2 + ["integer_to_percentage"])}
            }
        }
    }
    pub fn convert_transform(input: &[longhands::transform::computed_value::ComputedOperation],
                             output: &mut structs::root::RefPtr<structs::root::nsCSSValueSharedList>) {
        use gecko_bindings::sugar::refptr::RefPtr;

        unsafe { output.clear() };

        let list = unsafe {
            RefPtr::from_addrefed(bindings::Gecko_NewCSSValueSharedList(input.len() as u32))
        };
        let value_list = unsafe { list.mHead.as_mut() };
        if let Some(value_list) = value_list {
            for (gecko, servo) in value_list.into_iter().zip(input.into_iter()) {
                Self::set_single_transform_function(servo, gecko);
            }
        }
        unsafe { output.set_move(list) };
    }

    pub fn set_transform(&mut self, other: longhands::transform::computed_value::T) {
        let vec = if let Some(v) = other.0 {
            v
        } else {
            unsafe {
                self.gecko.mSpecifiedTransform.clear();
            }
            return;
        };
        Self::convert_transform(&vec, &mut self.gecko.mSpecifiedTransform);
    }

    pub fn copy_transform_from(&mut self, other: &Self) {
        unsafe { self.gecko.mSpecifiedTransform.set(&other.gecko.mSpecifiedTransform); }
    }

    <%def name="computed_operation_arm(name, keyword, items)">
        <%
            # %s is substituted with the call to GetArrayItem.
            css_value_getters = {
                "length" : "Au(bindings::Gecko_CSSValue_GetAbsoluteLength(%s))",
                "lop" : "%s.get_lop()",
                "angle" : "%s.get_angle()",
                "number" : "bindings::Gecko_CSSValue_GetNumber(%s)",
                "percentage" : "Percentage(bindings::Gecko_CSSValue_GetPercentage(%s))",
                "percentage_to_integer" : "bindings::Gecko_CSSValue_GetPercentage(%s) as i32",
                "list" : "TransformList(Some(convert_shared_list_to_operations(%s)))",
            }
            pre_symbols = "("
            post_symbols = ")"
            if keyword == "interpolatematrix" or keyword == "accumulatematrix":
                # We generate this like: "ComputedOperation::InterpolateMatrix {", so the space is
                # between "InterpolateMatrix"/"AccumulateMatrix" and '{'
                pre_symbols = " {"
                post_symbols = "}"
            elif keyword == "matrix3d":
                pre_symbols = "(ComputedMatrix {"
                post_symbols = "})"
            field_names = None
            if keyword == "interpolatematrix":
                field_names = ["from_list", "to_list", "progress"]
            elif keyword == "accumulatematrix":
                field_names = ["from_list", "to_list", "count"]
        %>
        structs::nsCSSKeyword::eCSSKeyword_${keyword} => {
            ComputedOperation::${name}${pre_symbols}
            % for index, item in enumerate(items):
                % if keyword == "matrix3d":
                    m${index / 4 + 1}${index % 4 + 1}:
                % elif keyword == "interpolatematrix" or keyword == "accumulatematrix":
                    ${field_names[index]}:
                % endif
                ${css_value_getters[item] % (
                    "bindings::Gecko_CSSValue_GetArrayItemConst(gecko_value, %d)" % (index + 1)
                )},
            % endfor
            ${post_symbols}
        },
    </%def>
    fn clone_single_transform_function(gecko_value: &structs::nsCSSValue)
                                       -> longhands::transform::computed_value::ComputedOperation {
        use properties::longhands::transform::computed_value::ComputedMatrix;
        use properties::longhands::transform::computed_value::ComputedOperation;
        use properties::longhands::transform::computed_value::T as TransformList;
        use values::computed::Percentage;

        let convert_shared_list_to_operations = |value: &structs::nsCSSValue|
                                                -> Vec<ComputedOperation> {
            debug_assert!(value.mUnit == structs::nsCSSUnit::eCSSUnit_SharedList);
            let value_list = unsafe {
                value.mValue.mSharedList.as_ref()
                     .as_mut().expect("List pointer should be non-null").mHead.as_ref()
            };
            debug_assert!(value_list.is_some(), "An empty shared list is not allowed");
            value_list.unwrap().into_iter()
                               .map(|item| Self::clone_single_transform_function(item))
                               .collect()
        };

        let transform_function = unsafe {
            bindings::Gecko_CSSValue_GetKeyword(bindings::Gecko_CSSValue_GetArrayItemConst(gecko_value, 0))
        };

        unsafe {
            match transform_function {
                ${computed_operation_arm("Matrix", "matrix3d", ["number"] * 16)}
                ${computed_operation_arm("Skew", "skew", ["angle"] * 2)}
                ${computed_operation_arm("Translate", "translate3d", ["lop", "lop", "length"])}
                ${computed_operation_arm("Scale", "scale3d", ["number"] * 3)}
                ${computed_operation_arm("Rotate", "rotate3d", ["number"] * 3 + ["angle"])}
                ${computed_operation_arm("Perspective", "perspective", ["length"])}
                ${computed_operation_arm("InterpolateMatrix", "interpolatematrix",
                                         ["list"] * 2 + ["percentage"])}
                ${computed_operation_arm("AccumulateMatrix", "accumulatematrix",
                                         ["list"] * 2 + ["percentage_to_integer"])}
                _ => panic!("We shouldn't set any other transform function types"),
            }
        }
    }
    pub fn clone_transform(&self) -> longhands::transform::computed_value::T {
        use properties::longhands::transform::computed_value;

        if self.gecko.mSpecifiedTransform.mRawPtr.is_null() {
            return computed_value::T(None);
        }
        let list = unsafe { (*self.gecko.mSpecifiedTransform.to_safe().get()).mHead.as_ref() };
        let result = list.map(|list| {
            list.into_iter()
                .map(|value| Self::clone_single_transform_function(value))
                .collect()
        });
        computed_value::T(result)
    }

    ${impl_transition_time_value('delay', 'Delay')}
    ${impl_transition_time_value('duration', 'Duration')}
    ${impl_transition_timing_function()}

    pub fn transition_combined_duration_at(&self, index: usize) -> f32 {
        // https://drafts.csswg.org/css-transitions/#transition-combined-duration
        self.gecko.mTransitions[index].mDuration.max(0.0) + self.gecko.mTransitions[index].mDelay
    }

    pub fn set_transition_property<I>(&mut self, v: I)
        where I: IntoIterator<Item = longhands::transition_property::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        use gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_no_properties;

        let v = v.into_iter();

        if v.len() != 0 {
            unsafe { self.gecko.mTransitions.ensure_len(v.len()) };
            self.gecko.mTransitionPropertyCount = v.len() as u32;
            for (servo, gecko) in v.zip(self.gecko.mTransitions.iter_mut()) {
                match servo {
                    TransitionProperty::Unsupported(ref ident) => unsafe {
                        Gecko_StyleTransition_SetUnsupportedProperty(gecko, ident.0.as_ptr())
                    },
                    _ => gecko.mProperty = (&servo).into(),
                }
            }
        } else {
            // In gecko |none| is represented by eCSSPropertyExtra_no_properties.
            self.gecko.mTransitionPropertyCount = 1;
            self.gecko.mTransitions[0].mProperty = eCSSPropertyExtra_no_properties;
        }
    }

    /// Returns whether there are any transitions specified.
    pub fn specifies_transitions(&self) -> bool {
        use gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_all_properties;
        if self.gecko.mTransitionPropertyCount == 1 &&
            self.gecko.mTransitions[0].mProperty == eCSSPropertyExtra_all_properties &&
            self.gecko.mTransitions[0].mDuration.max(0.0) + self.gecko.mTransitions[0].mDelay <= 0.0f32 {
            return false;
        }

        self.gecko.mTransitionPropertyCount > 0
    }

    pub fn transition_property_at(&self, index: usize)
        -> longhands::transition_property::computed_value::SingleComputedValue {
        use gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_no_properties;
        use gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_variable;
        use gecko_bindings::structs::nsCSSPropertyID::eCSSProperty_UNKNOWN;
        use gecko_bindings::structs::nsIAtom;

        let property = self.gecko.mTransitions[index].mProperty;
        if property == eCSSProperty_UNKNOWN || property == eCSSPropertyExtra_variable {
            let atom = self.gecko.mTransitions[index].mUnknownProperty.raw::<nsIAtom>();
            debug_assert!(!atom.is_null());
            TransitionProperty::Unsupported(CustomIdent(atom.into()))
        } else if property == eCSSPropertyExtra_no_properties {
            // Actually, we don't expect TransitionProperty::Unsupported also represents "none",
            // but if the caller wants to convert it, it is fine. Please use it carefully.
            TransitionProperty::Unsupported(CustomIdent(atom!("none")))
        } else {
            property.into()
        }
    }

    pub fn transition_nscsspropertyid_at(&self, index: usize) -> nsCSSPropertyID {
        self.gecko.mTransitions[index].mProperty
    }

    pub fn copy_transition_property_from(&mut self, other: &Self) {
        use gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_variable;
        use gecko_bindings::structs::nsCSSPropertyID::eCSSProperty_UNKNOWN;
        use gecko_bindings::structs::nsIAtom;
        unsafe { self.gecko.mTransitions.ensure_len(other.gecko.mTransitions.len()) };

        let count = other.gecko.mTransitionPropertyCount;
        self.gecko.mTransitionPropertyCount = count;

        for (index, transition) in self.gecko.mTransitions.iter_mut().enumerate().take(count as usize) {
            transition.mProperty = other.gecko.mTransitions[index].mProperty;
            if transition.mProperty == eCSSProperty_UNKNOWN ||
               transition.mProperty == eCSSPropertyExtra_variable {
                let atom = other.gecko.mTransitions[index].mUnknownProperty.raw::<nsIAtom>();
                debug_assert!(!atom.is_null());
                unsafe { Gecko_StyleTransition_SetUnsupportedProperty(transition, atom) };
            }
        }
    }
    ${impl_transition_count('property', 'Property')}

    pub fn animations_equals(&self, other: &Self) -> bool {
        unsafe { bindings::Gecko_StyleAnimationsEquals(&self.gecko.mAnimations, &other.gecko.mAnimations) }
    }

    pub fn set_animation_name<I>(&mut self, v: I)
        where I: IntoIterator<Item = longhands::animation_name::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {

        let v = v.into_iter();
        debug_assert!(v.len() != 0);
        unsafe { self.gecko.mAnimations.ensure_len(v.len()) };

        self.gecko.mAnimationNameCount = v.len() as u32;
        for (servo, gecko) in v.zip(self.gecko.mAnimations.iter_mut()) {
            // TODO This is inefficient. We should fix this in bug 1329169.
            gecko.mName.assign(match servo.0 {
                Some(ref name) => name.as_atom().as_slice(),
                None => &[],  // Empty string for 'none'
            });
        }
    }
    pub fn animation_name_at(&self, index: usize)
        -> longhands::animation_name::computed_value::SingleComputedValue {
        use properties::longhands::animation_name::single_value::SpecifiedValue as AnimationName;
        // XXX: Is there any effective ways?
        let atom = &self.gecko.mAnimations[index].mName;
        if atom.is_empty() {
            AnimationName(None)
        } else {
            AnimationName(Some(KeyframesName::from_ident(atom.to_string())))
        }
    }
    pub fn copy_animation_name_from(&mut self, other: &Self) {
        unsafe { self.gecko.mAnimations.ensure_len(other.gecko.mAnimations.len()) };

        let count = other.gecko.mAnimationNameCount;
        self.gecko.mAnimationNameCount = count;

        // The length of mAnimations is often greater than mAnimationXXCount,
        // don't copy values over the count.
        for (index, animation) in self.gecko.mAnimations.iter_mut().enumerate().take(count as usize) {
            animation.mName.assign(&*other.gecko.mAnimations[index].mName);
        }
    }
    ${impl_animation_count('name', 'Name')}

    ${impl_animation_time_value('delay', 'Delay')}
    ${impl_animation_time_value('duration', 'Duration')}

    ${impl_animation_keyword('direction', 'Direction',
                             data.longhands_by_name["animation-direction"].keyword)}
    ${impl_animation_keyword('fill_mode', 'FillMode',
                             data.longhands_by_name["animation-fill-mode"].keyword)}
    ${impl_animation_keyword('play_state', 'PlayState',
                             data.longhands_by_name["animation-play-state"].keyword)}

    pub fn set_animation_iteration_count<I>(&mut self, v: I)
        where I: IntoIterator<Item = longhands::animation_iteration_count::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator + Clone
    {
        use std::f32;
        use properties::longhands::animation_iteration_count::single_value::SpecifiedValue as AnimationIterationCount;

        let v = v.into_iter();

        debug_assert!(v.len() != 0);
        let input_len = v.len();
        unsafe { self.gecko.mAnimations.ensure_len(input_len) };

        self.gecko.mAnimationIterationCountCount = input_len as u32;
        for (gecko, servo) in self.gecko.mAnimations.iter_mut().zip(v.cycle()) {
            match servo {
                AnimationIterationCount::Number(n) => gecko.mIterationCount = n,
                AnimationIterationCount::Infinite => gecko.mIterationCount = f32::INFINITY,
            }
        }
    }
    pub fn animation_iteration_count_at(&self, index: usize)
        -> longhands::animation_iteration_count::computed_value::SingleComputedValue {
        use properties::longhands::animation_iteration_count::single_value::computed_value::T
            as AnimationIterationCount;

        if self.gecko.mAnimations[index].mIterationCount.is_infinite() {
            AnimationIterationCount::Infinite
        } else {
            AnimationIterationCount::Number(self.gecko.mAnimations[index].mIterationCount)
        }
    }
    ${impl_animation_count('iteration_count', 'IterationCount')}
    ${impl_copy_animation_value('iteration_count', 'IterationCount')}

    ${impl_animation_timing_function()}

    <% scroll_snap_type_keyword = Keyword("scroll-snap-type", "none mandatory proximity") %>

    ${impl_keyword('scroll_snap_type_y', 'mScrollSnapTypeY', scroll_snap_type_keyword, need_clone=True)}

    pub fn set_perspective_origin(&mut self, v: longhands::perspective_origin::computed_value::T) {
        self.gecko.mPerspectiveOrigin[0].set(v.horizontal);
        self.gecko.mPerspectiveOrigin[1].set(v.vertical);
    }

    pub fn copy_perspective_origin_from(&mut self, other: &Self) {
        self.gecko.mPerspectiveOrigin[0].copy_from(&other.gecko.mPerspectiveOrigin[0]);
        self.gecko.mPerspectiveOrigin[1].copy_from(&other.gecko.mPerspectiveOrigin[1]);
    }

    pub fn clone_perspective_origin(&self) -> longhands::perspective_origin::computed_value::T {
        use properties::longhands::perspective_origin::computed_value::T;
        use values::computed::LengthOrPercentage;
        T {
            horizontal: LengthOrPercentage::from_gecko_style_coord(&self.gecko.mPerspectiveOrigin[0])
                .expect("Expected length or percentage for horizontal value of perspective-origin"),
            vertical: LengthOrPercentage::from_gecko_style_coord(&self.gecko.mPerspectiveOrigin[1])
                .expect("Expected length or percentage for vertical value of perspective-origin"),
        }
    }

    pub fn set_transform_origin(&mut self, v: longhands::transform_origin::computed_value::T) {
        self.gecko.mTransformOrigin[0].set(v.horizontal);
        self.gecko.mTransformOrigin[1].set(v.vertical);
        self.gecko.mTransformOrigin[2].set(v.depth);
    }

    pub fn copy_transform_origin_from(&mut self, other: &Self) {
        self.gecko.mTransformOrigin[0].copy_from(&other.gecko.mTransformOrigin[0]);
        self.gecko.mTransformOrigin[1].copy_from(&other.gecko.mTransformOrigin[1]);
        self.gecko.mTransformOrigin[2].copy_from(&other.gecko.mTransformOrigin[2]);
    }

    pub fn clone_transform_origin(&self) -> longhands::transform_origin::computed_value::T {
        use properties::longhands::transform_origin::computed_value::T;
        use values::computed::LengthOrPercentage;
        T {
            horizontal: LengthOrPercentage::from_gecko_style_coord(&self.gecko.mTransformOrigin[0])
                .expect("clone for LengthOrPercentage failed"),
            vertical: LengthOrPercentage::from_gecko_style_coord(&self.gecko.mTransformOrigin[1])
                .expect("clone for LengthOrPercentage failed"),
            depth: Au::from_gecko_style_coord(&self.gecko.mTransformOrigin[2])
                .expect("clone for Length failed"),
        }
    }

    pub fn set_will_change(&mut self, v: longhands::will_change::computed_value::T) {
        use gecko_bindings::bindings::{Gecko_AppendWillChange, Gecko_ClearWillChange};
        use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_OPACITY;
        use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_SCROLL;
        use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_TRANSFORM;
        use properties::PropertyId;
        use properties::longhands::will_change::computed_value::T;

        fn will_change_bitfield_from_prop_flags(prop: &LonghandId) -> u8 {
            use properties::{ABSPOS_CB, CREATES_STACKING_CONTEXT, FIXPOS_CB};
            use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_ABSPOS_CB;
            use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_FIXPOS_CB;
            use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_STACKING_CONTEXT;
            let servo_flags = prop.flags();
            let mut bitfield = 0;

            if servo_flags.contains(CREATES_STACKING_CONTEXT) {
                bitfield |= NS_STYLE_WILL_CHANGE_STACKING_CONTEXT;
            }
            if servo_flags.contains(FIXPOS_CB) {
                bitfield |= NS_STYLE_WILL_CHANGE_FIXPOS_CB;
            }
            if servo_flags.contains(ABSPOS_CB) {
                bitfield |= NS_STYLE_WILL_CHANGE_ABSPOS_CB;
            }

            bitfield as u8
        }

        self.gecko.mWillChangeBitField = 0;

        match v {
            T::AnimateableFeatures(features) => {
                unsafe {
                    Gecko_ClearWillChange(&mut self.gecko, features.len());
                }

                for feature in features.iter() {
                    if feature.0 == atom!("scroll-position") {
                        self.gecko.mWillChangeBitField |= NS_STYLE_WILL_CHANGE_SCROLL as u8;
                    } else if feature.0 == atom!("opacity") {
                        self.gecko.mWillChangeBitField |= NS_STYLE_WILL_CHANGE_OPACITY as u8;
                    } else if feature.0 == atom!("transform") {
                        self.gecko.mWillChangeBitField |= NS_STYLE_WILL_CHANGE_TRANSFORM as u8;
                    }

                    unsafe {
                        Gecko_AppendWillChange(&mut self.gecko, feature.0.as_ptr());
                    }

                    if let Ok(prop_id) = PropertyId::parse(feature.0.to_string().into()) {
                        match prop_id.as_shorthand() {
                            Ok(shorthand) => {
                                for longhand in shorthand.longhands() {
                                    self.gecko.mWillChangeBitField |=
                                        will_change_bitfield_from_prop_flags(longhand);
                                }
                            },
                            Err(longhand_or_custom) => {
                                if let PropertyDeclarationId::Longhand(longhand)
                                    = longhand_or_custom {
                                    self.gecko.mWillChangeBitField |=
                                        will_change_bitfield_from_prop_flags(&longhand);
                                }
                            },
                        }
                    }
                }
            },
            T::Auto => {
                unsafe {
                    Gecko_ClearWillChange(&mut self.gecko, 0);
                }
            },
        };
    }

    pub fn copy_will_change_from(&mut self, other: &Self) {
        use gecko_bindings::bindings::Gecko_CopyWillChangeFrom;

        self.gecko.mWillChangeBitField = other.gecko.mWillChangeBitField;
        unsafe {
            Gecko_CopyWillChangeFrom(&mut self.gecko, &other.gecko as *const _ as *mut _);
        }
    }

    pub fn clone_will_change(&self) -> longhands::will_change::computed_value::T {
        use properties::longhands::will_change::computed_value::T;
        use gecko_bindings::structs::nsIAtom;
        use gecko_string_cache::Atom;
        use values::CustomIdent;

        if self.gecko.mWillChange.mBuffer.len() == 0 {
            T::Auto
        } else {
            T::AnimateableFeatures(
                self.gecko.mWillChange.mBuffer.iter().map(|gecko_atom| {
                    CustomIdent(
                        unsafe { Atom::from_addrefed(*gecko_atom as *mut nsIAtom) }
                    )
                }).collect()
            )
        }
    }

    <% impl_shape_source("shape_outside", "mShapeOutside") %>

    pub fn set_contain(&mut self, v: longhands::contain::computed_value::T) {
        use gecko_bindings::structs::NS_STYLE_CONTAIN_NONE;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_STRICT;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_LAYOUT;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_STYLE;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_PAINT;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_ALL_BITS;
        use properties::longhands::contain;

        if v.is_empty() {
            self.gecko.mContain = NS_STYLE_CONTAIN_NONE as u8;
            return;
        }

        if v.contains(contain::STRICT) {
            self.gecko.mContain = (NS_STYLE_CONTAIN_STRICT | NS_STYLE_CONTAIN_ALL_BITS) as u8;
            return;
        }

        let mut bitfield = 0;
        if v.contains(contain::LAYOUT) {
            bitfield |= NS_STYLE_CONTAIN_LAYOUT;
        }
        if v.contains(contain::STYLE) {
            bitfield |= NS_STYLE_CONTAIN_STYLE;
        }
        if v.contains(contain::PAINT) {
            bitfield |= NS_STYLE_CONTAIN_PAINT;
        }

        self.gecko.mContain = bitfield as u8;
    }

    pub fn clone_contain(&self) -> longhands::contain::computed_value::T {
        use gecko_bindings::structs::NS_STYLE_CONTAIN_STRICT;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_LAYOUT;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_STYLE;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_PAINT;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_ALL_BITS;
        use properties::longhands::contain;

        let mut servo_flags = contain::computed_value::T::empty();
        let gecko_flags = self.gecko.mContain;

        if gecko_flags & (NS_STYLE_CONTAIN_STRICT as u8) != 0 &&
           gecko_flags & (NS_STYLE_CONTAIN_ALL_BITS as u8) != 0 {
            servo_flags.insert(contain::STRICT | contain::STRICT_BITS);
            return servo_flags;
        }

        if gecko_flags & (NS_STYLE_CONTAIN_LAYOUT as u8) != 0 {
            servo_flags.insert(contain::LAYOUT);
        }
        if gecko_flags & (NS_STYLE_CONTAIN_STYLE as u8) != 0{
            servo_flags.insert(contain::STYLE);
        }
        if gecko_flags & (NS_STYLE_CONTAIN_PAINT as u8) != 0 {
            servo_flags.insert(contain::PAINT);
        }

        return servo_flags;
    }

    ${impl_simple_copy("contain", "mContain")}

    ${impl_simple_type_with_conversion("touch_action")}
</%self:impl_trait>

<%def name="simple_image_array_property(name, shorthand, field_name)">
    <%
        image_layers_field = "mImage" if shorthand == "background" else "mMask"
        copy_simple_image_array_property(name, shorthand, image_layers_field, field_name)
    %>

    pub fn set_${shorthand}_${name}<I>(&mut self, v: I)
        where I: IntoIterator<Item=longhands::${shorthand}_${name}::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;
        let v = v.into_iter();

        unsafe {
          Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field}, v.len(),
                                        LayerType::${shorthand.title()});
        }

        self.gecko.${image_layers_field}.${field_name}Count = v.len() as u32;
        for (servo, geckolayer) in v.zip(self.gecko.${image_layers_field}.mLayers.iter_mut()) {
            geckolayer.${field_name} = {
                ${caller.body()}
            };
        }
    }
</%def>

<%def name="copy_simple_image_array_property(name, shorthand, layers_field_name, field_name)">
    pub fn copy_${shorthand}_${name}_from(&mut self, other: &Self) {
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        let count = other.gecko.${layers_field_name}.${field_name}Count;
        unsafe {
            Gecko_EnsureImageLayersLength(&mut self.gecko.${layers_field_name},
                                          count as usize,
                                          LayerType::${shorthand.title()});
        }
        for (layer, other) in self.gecko.${layers_field_name}.mLayers.iter_mut()
                                  .zip(other.gecko.${layers_field_name}.mLayers.iter())
                                  .take(count as usize) {
            layer.${field_name} = other.${field_name};
        }
        self.gecko.${layers_field_name}.${field_name}Count = count;
    }
</%def>

<%def name="impl_simple_image_array_property(name, shorthand, layer_field_name, field_name, struct_name)">
    <%
        ident = "%s_%s" % (shorthand, name)
        style_struct = next(x for x in data.style_structs if x.name == struct_name)
        longhand = next(x for x in style_struct.longhands if x.ident == ident)
        keyword = longhand.keyword
    %>

    <% copy_simple_image_array_property(name, shorthand, layer_field_name, field_name) %>

    pub fn set_${ident}<I>(&mut self, v: I)
        where I: IntoIterator<Item=longhands::${ident}::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        use properties::longhands::${ident}::single_value::computed_value::T as Keyword;
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        let v = v.into_iter();

        unsafe {
          Gecko_EnsureImageLayersLength(&mut self.gecko.${layer_field_name}, v.len(),
                                        LayerType::${shorthand.title()});
        }

        self.gecko.${layer_field_name}.${field_name}Count = v.len() as u32;
        for (servo, geckolayer) in v.zip(self.gecko.${layer_field_name}.mLayers.iter_mut()) {
            geckolayer.${field_name} = {
                match servo {
                    % for value in keyword.values_for("gecko"):
                    Keyword::${to_rust_ident(value)} =>
                        structs::${keyword.gecko_constant(value)} ${keyword.maybe_cast('u8')},
                    % endfor
                }
            };
        }
    }

    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T
    {
        use properties::longhands::${ident}::single_value::computed_value::T as Keyword;

        % if keyword.needs_cast():
        % for value in keyword.values_for('gecko'):
        const ${keyword.casted_constant_name(value, "u8")} : u8 =
            structs::${keyword.gecko_constant(value)} as u8;
        % endfor
        % endif

        longhands::${ident}::computed_value::T (
            self.gecko.${layer_field_name}.mLayers.iter()
                .take(self.gecko.${layer_field_name}.${field_name}Count as usize)
                .map(|ref layer| {
                    match layer.${field_name} {
                        % for value in longhand.keyword.values_for("gecko"):
                        % if keyword.needs_cast():
                        ${keyword.casted_constant_name(value, "u8")}
                        % else:
                        structs::${keyword.gecko_constant(value)}
                        % endif
                            => Keyword::${to_rust_ident(value)},
                        % endfor
                        x => panic!("Found unexpected value in style struct for ${ident} property: {:?}", x),
                    }
                }).collect()
        )
    }
</%def>

<%def name="impl_common_image_layer_properties(shorthand)">
    <%
        if shorthand == "background":
            image_layers_field = "mImage"
            struct_name = "Background"
        else:
            image_layers_field = "mMask"
            struct_name = "SVG"
    %>

    <%self:simple_image_array_property name="repeat" shorthand="${shorthand}" field_name="mRepeat">
        use properties::longhands::${shorthand}_repeat::single_value::computed_value::RepeatKeyword;
        use gecko_bindings::structs::nsStyleImageLayers_Repeat;
        use gecko_bindings::structs::StyleImageLayerRepeat;

        fn to_ns(repeat: RepeatKeyword) -> StyleImageLayerRepeat {
            match repeat {
                RepeatKeyword::Repeat => StyleImageLayerRepeat::Repeat,
                RepeatKeyword::Space => StyleImageLayerRepeat::Space,
                RepeatKeyword::Round => StyleImageLayerRepeat::Round,
                RepeatKeyword::NoRepeat => StyleImageLayerRepeat::NoRepeat,
            }
        }

        let repeat_x = to_ns(servo.0);
        let repeat_y = to_ns(servo.1);
        nsStyleImageLayers_Repeat {
              mXRepeat: repeat_x,
              mYRepeat: repeat_y,
        }
    </%self:simple_image_array_property>

    pub fn clone_${shorthand}_repeat(&self) -> longhands::${shorthand}_repeat::computed_value::T {
        use properties::longhands::${shorthand}_repeat::single_value::computed_value::T;
        use properties::longhands::${shorthand}_repeat::single_value::computed_value::RepeatKeyword;
        use gecko_bindings::structs::StyleImageLayerRepeat;

        fn to_servo(repeat: StyleImageLayerRepeat) -> RepeatKeyword {
            match repeat {
                StyleImageLayerRepeat::Repeat => RepeatKeyword::Repeat,
                StyleImageLayerRepeat::Space => RepeatKeyword::Space,
                StyleImageLayerRepeat::Round => RepeatKeyword::Round,
                StyleImageLayerRepeat::NoRepeat => RepeatKeyword::NoRepeat,
                x => panic!("Found unexpected value in style struct for ${shorthand}_repeat property: {:?}", x),
            }
        }

        longhands::${shorthand}_repeat::computed_value::T (
            self.gecko.${image_layers_field}.mLayers.iter()
                .take(self.gecko.${image_layers_field}.mRepeatCount as usize)
                .map(|ref layer| {
                    T(to_servo(layer.mRepeat.mXRepeat), to_servo(layer.mRepeat.mYRepeat))
                }).collect()
        )
    }

    <% impl_simple_image_array_property("clip", shorthand, image_layers_field, "mClip", struct_name) %>
    <% impl_simple_image_array_property("origin", shorthand, image_layers_field, "mOrigin", struct_name) %>

    % for orientation in ["x", "y"]:
    pub fn copy_${shorthand}_position_${orientation}_from(&mut self, other: &Self) {
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        let count = other.gecko.${image_layers_field}.mPosition${orientation.upper()}Count;

        unsafe {
            Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field},
                                          count as usize,
                                          LayerType::${shorthand.capitalize()});
        }

        for (layer, other) in self.gecko.${image_layers_field}.mLayers.iter_mut()
                                  .zip(other.gecko.${image_layers_field}.mLayers.iter())
                                  .take(count as usize) {
            layer.mPosition.m${orientation.upper()}Position
                = other.mPosition.m${orientation.upper()}Position;
        }
        self.gecko.${image_layers_field}.mPosition${orientation.upper()}Count = count;
    }

    pub fn clone_${shorthand}_position_${orientation}(&self)
        -> longhands::${shorthand}_position_${orientation}::computed_value::T {
        longhands::${shorthand}_position_${orientation}::computed_value::T(
            self.gecko.${image_layers_field}.mLayers.iter()
                .take(self.gecko.${image_layers_field}.mPosition${orientation.upper()}Count as usize)
                .map(|position| position.mPosition.m${orientation.upper()}Position.into())
                .collect()
        )
    }

    pub fn set_${shorthand}_position_${orientation[0]}<I>(&mut self,
                                     v: I)
        where I: IntoIterator<Item = longhands::${shorthand}_position_${orientation[0]}
                                              ::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        let v = v.into_iter();

        unsafe {
            Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field}, v.len(),
                                        LayerType::${shorthand.capitalize()});
        }

        self.gecko.${image_layers_field}.mPosition${orientation[0].upper()}Count = v.len() as u32;
        for (servo, geckolayer) in v.zip(self.gecko.${image_layers_field}
                                                           .mLayers.iter_mut()) {
            geckolayer.mPosition.m${orientation[0].upper()}Position = servo.into();
        }
    }
    % endfor

    <%self:simple_image_array_property name="size" shorthand="${shorthand}" field_name="mSize">
        use gecko_bindings::structs::nsStyleImageLayers_Size_Dimension;
        use gecko_bindings::structs::nsStyleImageLayers_Size_DimensionType;
        use gecko_bindings::structs::{nsStyleCoord_CalcValue, nsStyleImageLayers_Size};
        use values::generics::background::BackgroundSize;

        let mut width = nsStyleCoord_CalcValue::new();
        let mut height = nsStyleCoord_CalcValue::new();

        let (w_type, h_type) = match servo {
            BackgroundSize::Explicit { width: explicit_width, height: explicit_height } => {
                let mut w_type = nsStyleImageLayers_Size_DimensionType::eAuto;
                let mut h_type = nsStyleImageLayers_Size_DimensionType::eAuto;
                if let Some(w) = explicit_width.to_calc_value() {
                    width = w;
                    w_type = nsStyleImageLayers_Size_DimensionType::eLengthPercentage;
                }
                if let Some(h) = explicit_height.to_calc_value() {
                    height = h;
                    h_type = nsStyleImageLayers_Size_DimensionType::eLengthPercentage;
                }
                (w_type, h_type)
            }
            BackgroundSize::Cover => {
                (
                    nsStyleImageLayers_Size_DimensionType::eCover,
                    nsStyleImageLayers_Size_DimensionType::eCover,
                )
            },
            BackgroundSize::Contain => {
                (
                    nsStyleImageLayers_Size_DimensionType::eContain,
                    nsStyleImageLayers_Size_DimensionType::eContain,
                )
            },
        };

        nsStyleImageLayers_Size {
            mWidth: nsStyleImageLayers_Size_Dimension { _base: width },
            mHeight: nsStyleImageLayers_Size_Dimension { _base: height },
            mWidthType: w_type as u8,
            mHeightType: h_type as u8,
        }
    </%self:simple_image_array_property>

    pub fn clone_${shorthand}_size(&self) -> longhands::background_size::computed_value::T {
        use gecko_bindings::structs::nsStyleCoord_CalcValue as CalcValue;
        use gecko_bindings::structs::nsStyleImageLayers_Size_DimensionType as DimensionType;
        use values::computed::LengthOrPercentageOrAuto;
        use values::generics::background::BackgroundSize;

        fn to_servo(value: CalcValue, ty: u8) -> LengthOrPercentageOrAuto {
            if ty == DimensionType::eAuto as u8 {
                LengthOrPercentageOrAuto::Auto
            } else {
                debug_assert!(ty == DimensionType::eLengthPercentage as u8);
                LengthOrPercentageOrAuto::Calc(value.into())
            }
        }

        longhands::background_size::computed_value::T(
            self.gecko.${image_layers_field}.mLayers.iter().map(|ref layer| {
                if DimensionType::eCover as u8 == layer.mSize.mWidthType {
                    debug_assert!(layer.mSize.mHeightType == DimensionType::eCover as u8);
                    return BackgroundSize::Cover
                }
                if DimensionType::eContain as u8 == layer.mSize.mWidthType {
                    debug_assert!(layer.mSize.mHeightType == DimensionType::eContain as u8);
                    return BackgroundSize::Contain
                }
                BackgroundSize::Explicit {
                    width: to_servo(layer.mSize.mWidth._base, layer.mSize.mWidthType),
                    height: to_servo(layer.mSize.mHeight._base, layer.mSize.mHeightType),
                }
            }).collect()
        )
    }


    pub fn copy_${shorthand}_image_from(&mut self, other: &Self) {
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;
        unsafe {
            let count = other.gecko.${image_layers_field}.mImageCount;
            unsafe {
                Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field},
                                              count as usize,
                                              LayerType::${shorthand.capitalize()});
            }

            for (layer, other) in self.gecko.${image_layers_field}.mLayers.iter_mut()
                                      .zip(other.gecko.${image_layers_field}.mLayers.iter())
                                      .take(count as usize) {
                Gecko_CopyImageValueFrom(&mut layer.mImage, &other.mImage);
            }
            self.gecko.${image_layers_field}.mImageCount = count;
        }
    }

    #[allow(unused_variables)]
    pub fn set_${shorthand}_image<I>(&mut self, images: I, cacheable: &mut bool)
        where I: IntoIterator<Item = longhands::${shorthand}_image::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        let images = images.into_iter();

        unsafe {
            // Prevent leaking of the last elements we did set
            for image in &mut self.gecko.${image_layers_field}.mLayers {
                Gecko_SetNullImageValue(&mut image.mImage)
            }
            // XXXManishearth clear mSourceURI for masks
            Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field}, images.len(),
                                          LayerType::${shorthand.title()});
        }

        self.gecko.${image_layers_field}.mImageCount = images.len() as u32;

        for (image, geckoimage) in images.zip(self.gecko.${image_layers_field}
                                                  .mLayers.iter_mut()) {
            if let Either::Second(image) = image {
                geckoimage.mImage.set(image, cacheable)
            }
        }
    }

    pub fn clone_${shorthand}_image(&self) -> longhands::${shorthand}_image::computed_value::T {
        use values::None_;

        longhands::${shorthand}_image::computed_value::T(
            self.gecko.${image_layers_field}.mLayers.iter()
                .take(self.gecko.${image_layers_field}.mImageCount as usize)
                .map(|ref layer| {
                    match unsafe { layer.mImage.into_image() } {
                        Some(image) => Either::Second(image),
                        None => Either::First(None_),
                    }
            }).collect()
        )
    }

    <%
        fill_fields = "mRepeat mClip mOrigin mPositionX mPositionY mImage mSize"
        if shorthand == "background":
            fill_fields += " mAttachment mBlendMode"
        else:
            # mSourceURI uses mImageCount
            fill_fields += " mMaskMode mComposite"
    %>
    pub fn fill_arrays(&mut self) {
        use gecko_bindings::bindings::Gecko_FillAll${shorthand.title()}Lists;
        use std::cmp;
        let mut max_len = 1;
        % for member in fill_fields.split():
            max_len = cmp::max(max_len, self.gecko.${image_layers_field}.${member}Count);
        % endfor
        unsafe {
            // While we could do this manually, we'd need to also manually
            // run all the copy constructors, so we just delegate to gecko
            Gecko_FillAll${shorthand.title()}Lists(&mut self.gecko.${image_layers_field}, max_len);
        }
    }
</%def>

// TODO: Gecko accepts lists in most background-related properties. We just use
// the first element (which is the common case), but at some point we want to
// add support for parsing these lists in servo and pushing to nsTArray's.
<% skip_background_longhands = """background-repeat
                                  background-image background-clip
                                  background-origin background-attachment
                                  background-size background-position
                                  background-blend-mode
                                  background-position-x
                                  background-position-y""" %>
<%self:impl_trait style_struct_name="Background"
                  skip_longhands="${skip_background_longhands}"
                  skip_additionals="*">

    <% impl_common_image_layer_properties("background") %>
    <% impl_simple_image_array_property("attachment", "background", "mImage", "mAttachment", "Background") %>
    <% impl_simple_image_array_property("blend_mode", "background", "mImage", "mBlendMode", "Background") %>
</%self:impl_trait>

<%self:impl_trait style_struct_name="List"
                  skip_longhands="list-style-image list-style-type quotes -moz-image-region"
                  skip_additionals="*">

    pub fn set_list_style_image(&mut self, image: longhands::list_style_image::computed_value::T) {
        use values::Either;
        match image {
            longhands::list_style_image::computed_value::T(Either::Second(_none)) => {
                unsafe {
                    Gecko_SetListStyleImageNone(&mut self.gecko);
                }
            }
            longhands::list_style_image::computed_value::T(Either::First(ref url)) => {
                unsafe {
                    Gecko_SetListStyleImageImageValue(&mut self.gecko,
                                                      url.image_value.clone().unwrap().get());
                }
                // We don't need to record this struct as uncacheable, like when setting
                // background-image to a url() value, since only properties in reset structs
                // are re-used from the applicable declaration cache, and the List struct
                // is an inherited struct.
            }
        }
    }

    pub fn copy_list_style_image_from(&mut self, other: &Self) {
        unsafe { Gecko_CopyListStyleImageFrom(&mut self.gecko, &other.gecko); }
    }

    pub fn clone_list_style_image(&self) -> longhands::list_style_image::computed_value::T {
        use values::specified::url::SpecifiedUrl;
        use values::{Either, None_};

        longhands::list_style_image::computed_value::T(
            match self.gecko.mListStyleImage.mRawPtr.is_null() {
                true => Either::Second(None_),
                false => {
                    unsafe {
                        let ref gecko_image_request = *self.gecko.mListStyleImage.mRawPtr;
                        Either::First(SpecifiedUrl::from_image_request(gecko_image_request)
                                      .expect("mListStyleImage could not convert to SpecifiedUrl"))
                    }
                }
            }
        )
    }

    pub fn set_list_style_type(&mut self, v: longhands::list_style_type::computed_value::T, device: &Device) {
        use gecko_bindings::bindings::Gecko_SetCounterStyleToString;
        use nsstring::{nsACString, nsCString};
        use self::longhands::list_style_type::computed_value::T;
        match v {
            T::CounterStyle(s) => s.to_gecko_value(&mut self.gecko.mCounterStyle, device),
            T::String(s) => unsafe {
                Gecko_SetCounterStyleToString(&mut self.gecko.mCounterStyle,
                                              &nsCString::from(s) as &nsACString)
            }
        }
    }

    pub fn copy_list_style_type_from(&mut self, other: &Self) {
        unsafe {
            Gecko_CopyCounterStyle(&mut self.gecko.mCounterStyle, &other.gecko.mCounterStyle);
        }
    }

    pub fn set_quotes(&mut self, other: longhands::quotes::computed_value::T) {
        use gecko_bindings::bindings::Gecko_NewStyleQuoteValues;
        use gecko_bindings::sugar::refptr::UniqueRefPtr;

        let mut refptr = unsafe {
            UniqueRefPtr::from_addrefed(Gecko_NewStyleQuoteValues(other.0.len() as u32))
        };

        for (servo, gecko) in other.0.into_iter().zip(refptr.mQuotePairs.iter_mut()) {
            gecko.first.assign_utf8(&servo.0);
            gecko.second.assign_utf8(&servo.1);
        }

        unsafe { self.gecko.mQuotes.set_move(refptr.get()) }
    }

    pub fn copy_quotes_from(&mut self, other: &Self) {
        unsafe { self.gecko.mQuotes.set(&other.gecko.mQuotes); }
    }

    pub fn clone_quotes(&self) -> longhands::quotes::computed_value::T {
        unsafe {
            let ref gecko_quote_values = *self.gecko.mQuotes.mRawPtr;
            longhands::quotes::computed_value::T(
                gecko_quote_values.mQuotePairs.iter().map(|gecko_pair| {
                    (gecko_pair.first.to_string(), gecko_pair.second.to_string())
                }).collect()
            )
        }
    }

    #[allow(non_snake_case)]
    pub fn set__moz_image_region(&mut self, v: longhands::_moz_image_region::computed_value::T) {
        use values::Either;

        match v {
            Either::Second(_auto) => {
                self.gecko.mImageRegion.x = 0;
                self.gecko.mImageRegion.y = 0;
                self.gecko.mImageRegion.width = 0;
                self.gecko.mImageRegion.height = 0;
            }
            Either::First(rect) => {
                self.gecko.mImageRegion.x = rect.left.unwrap_or(Au(0)).0;
                self.gecko.mImageRegion.y = rect.top.unwrap_or(Au(0)).0;
                self.gecko.mImageRegion.height = match rect.bottom {
                    Some(value) => value.0 - self.gecko.mImageRegion.y,
                    None => 0,
                };
                self.gecko.mImageRegion.width = match rect.right {
                    Some(value) => value.0 - self.gecko.mImageRegion.x,
                    None => 0,
                };
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn clone__moz_image_region(&self) -> longhands::_moz_image_region::computed_value::T {
        use values::{Auto, Either};
        use values::computed::ClipRect;

        // There is no ideal way to detect auto type for structs::nsRect and its components, so
        // if all components are zero, we use Auto.
        if self.gecko.mImageRegion.x == 0 &&
           self.gecko.mImageRegion.y == 0 &&
           self.gecko.mImageRegion.width == 0 &&
           self.gecko.mImageRegion.height == 0 {
           return Either::Second(Auto);
        }

        let get_clip_rect_component = |value: structs::nscoord| -> Option<Au> {
            if value == 0 {
                None
            } else {
                Some(Au(value))
            }
        };

        Either::First(ClipRect {
            top: get_clip_rect_component(self.gecko.mImageRegion.y),
            right: get_clip_rect_component(self.gecko.mImageRegion.width).map(
                |v| v + Au(self.gecko.mImageRegion.x)),
            bottom: get_clip_rect_component(self.gecko.mImageRegion.height).map(
                |v| v + Au(self.gecko.mImageRegion.y)),
            left: get_clip_rect_component(self.gecko.mImageRegion.x),
        })
    }

    ${impl_simple_copy('_moz_image_region', 'mImageRegion')}

</%self:impl_trait>

<%self:impl_trait style_struct_name="Table" skip_longhands="-x-span">
    #[allow(non_snake_case)]
    pub fn set__x_span(&mut self, v: longhands::_x_span::computed_value::T) {
        self.gecko.mSpan = v.0
    }

    ${impl_simple_copy('_x_span', 'mSpan')}
</%self:impl_trait>

<%self:impl_trait style_struct_name="Effects"
                  skip_longhands="box-shadow clip filter">
    pub fn set_box_shadow<I>(&mut self, v: I)
        where I: IntoIterator<Item = BoxShadow>,
              I::IntoIter: ExactSizeIterator
    {
        let v = v.into_iter();
        self.gecko.mBoxShadow.replace_with_new(v.len() as u32);
        for (servo, gecko_shadow) in v.zip(self.gecko.mBoxShadow.iter_mut()) {
            gecko_shadow.set_from_box_shadow(servo);
        }
    }

    pub fn copy_box_shadow_from(&mut self, other: &Self) {
        self.gecko.mBoxShadow.copy_from(&other.gecko.mBoxShadow);
    }

    pub fn clone_box_shadow(&self) -> longhands::box_shadow::computed_value::T {
        let buf = self.gecko.mBoxShadow.iter().map(|v| v.to_box_shadow()).collect();
        longhands::box_shadow::computed_value::T(buf)
    }

    pub fn set_clip(&mut self, v: longhands::clip::computed_value::T) {
        use gecko_bindings::structs::NS_STYLE_CLIP_AUTO;
        use gecko_bindings::structs::NS_STYLE_CLIP_RECT;
        use gecko_bindings::structs::NS_STYLE_CLIP_LEFT_AUTO;
        use gecko_bindings::structs::NS_STYLE_CLIP_TOP_AUTO;
        use gecko_bindings::structs::NS_STYLE_CLIP_RIGHT_AUTO;
        use gecko_bindings::structs::NS_STYLE_CLIP_BOTTOM_AUTO;
        use values::Either;

        match v {
            Either::First(rect) => {
                self.gecko.mClipFlags = NS_STYLE_CLIP_RECT as u8;
                if let Some(left) = rect.left {
                    self.gecko.mClip.x = left.0;
                } else {
                    self.gecko.mClip.x = 0;
                    self.gecko.mClipFlags |= NS_STYLE_CLIP_LEFT_AUTO as u8;
                }

                if let Some(top) = rect.top {
                    self.gecko.mClip.y = top.0;
                } else {
                    self.gecko.mClip.y = 0;
                    self.gecko.mClipFlags |= NS_STYLE_CLIP_TOP_AUTO as u8;
                }

                if let Some(bottom) = rect.bottom {
                    self.gecko.mClip.height = bottom.0 - self.gecko.mClip.y;
                } else {
                    self.gecko.mClip.height = 1 << 30; // NS_MAXSIZE
                    self.gecko.mClipFlags |= NS_STYLE_CLIP_BOTTOM_AUTO as u8;
                }

                if let Some(right) = rect.right {
                    self.gecko.mClip.width = right.0 - self.gecko.mClip.x;
                } else {
                    self.gecko.mClip.width = 1 << 30; // NS_MAXSIZE
                    self.gecko.mClipFlags |= NS_STYLE_CLIP_RIGHT_AUTO as u8;
                }
            },
            Either::Second(_auto) => {
                self.gecko.mClipFlags = NS_STYLE_CLIP_AUTO as u8;
                self.gecko.mClip.x = 0;
                self.gecko.mClip.y = 0;
                self.gecko.mClip.width = 0;
                self.gecko.mClip.height = 0;
            }
        }
    }

    pub fn copy_clip_from(&mut self, other: &Self) {
        self.gecko.mClip = other.gecko.mClip;
        self.gecko.mClipFlags = other.gecko.mClipFlags;
    }

    pub fn clone_clip(&self) -> longhands::clip::computed_value::T {
        use gecko_bindings::structs::NS_STYLE_CLIP_AUTO;
        use gecko_bindings::structs::NS_STYLE_CLIP_BOTTOM_AUTO;
        use gecko_bindings::structs::NS_STYLE_CLIP_LEFT_AUTO;
        use gecko_bindings::structs::NS_STYLE_CLIP_RIGHT_AUTO;
        use gecko_bindings::structs::NS_STYLE_CLIP_TOP_AUTO;
        use values::computed::{ClipRect, ClipRectOrAuto};
        use values::Either;

        if self.gecko.mClipFlags == NS_STYLE_CLIP_AUTO as u8 {
            ClipRectOrAuto::auto()
        } else {
            let left = if self.gecko.mClipFlags & NS_STYLE_CLIP_LEFT_AUTO as u8 != 0 {
                debug_assert!(self.gecko.mClip.x == 0);
                None
            } else {
                Some(Au(self.gecko.mClip.x))
            };

            let top = if self.gecko.mClipFlags & NS_STYLE_CLIP_TOP_AUTO as u8 != 0 {
                debug_assert!(self.gecko.mClip.y == 0);
                None
            } else {
                Some(Au(self.gecko.mClip.y))
            };

            let bottom = if self.gecko.mClipFlags & NS_STYLE_CLIP_BOTTOM_AUTO as u8 != 0 {
                debug_assert!(self.gecko.mClip.height == 1 << 30); // NS_MAXSIZE
                None
            } else {
                Some(Au(self.gecko.mClip.y + self.gecko.mClip.height))
            };

            let right = if self.gecko.mClipFlags & NS_STYLE_CLIP_RIGHT_AUTO as u8 != 0 {
                debug_assert!(self.gecko.mClip.width == 1 << 30); // NS_MAXSIZE
                None
            } else {
                Some(Au(self.gecko.mClip.x + self.gecko.mClip.width))
            };

            Either::First(ClipRect { top: top, right: right, bottom: bottom, left: left, })
        }
    }

    <%
    # This array is several filter function which has percentage or
    # number value for function of clone / set.
    # The setting / cloning process of other function(e.g. Blur / HueRotate) is
    # different from these function. So this array don't include such function.
    FILTER_FUNCTIONS = [ 'Brightness', 'Contrast', 'Grayscale', 'Invert',
                         'Opacity', 'Saturate', 'Sepia' ]
     %>

    pub fn set_filter<I>(&mut self, v: I)
    where
        I: IntoIterator<Item = Filter>,
        I::IntoIter: ExactSizeIterator,
    {
        use values::generics::effects::Filter::*;
        use gecko_bindings::structs::nsCSSShadowArray;
        use gecko_bindings::structs::nsStyleFilter;
        use gecko_bindings::structs::NS_STYLE_FILTER_BLUR;
        use gecko_bindings::structs::NS_STYLE_FILTER_BRIGHTNESS;
        use gecko_bindings::structs::NS_STYLE_FILTER_CONTRAST;
        use gecko_bindings::structs::NS_STYLE_FILTER_GRAYSCALE;
        use gecko_bindings::structs::NS_STYLE_FILTER_INVERT;
        use gecko_bindings::structs::NS_STYLE_FILTER_OPACITY;
        use gecko_bindings::structs::NS_STYLE_FILTER_SATURATE;
        use gecko_bindings::structs::NS_STYLE_FILTER_SEPIA;
        use gecko_bindings::structs::NS_STYLE_FILTER_HUE_ROTATE;
        use gecko_bindings::structs::NS_STYLE_FILTER_DROP_SHADOW;

        fn fill_filter(m_type: u32, value: CoordDataValue, gecko_filter: &mut nsStyleFilter){
            gecko_filter.mType = m_type;
            gecko_filter.mFilterParameter.set_value(value);
        }

        let v = v.into_iter();
        unsafe {
            Gecko_ResetFilters(&mut self.gecko, v.len());
        }
        debug_assert_eq!(v.len(), self.gecko.mFilters.len());

        for (servo, gecko_filter) in v.zip(self.gecko.mFilters.iter_mut()) {
            match servo {
                % for func in FILTER_FUNCTIONS:
                ${func}(factor) => fill_filter(NS_STYLE_FILTER_${func.upper()},
                                               CoordDataValue::Factor(factor),
                                               gecko_filter),
                % endfor
                Blur(length) => fill_filter(NS_STYLE_FILTER_BLUR,
                                            CoordDataValue::Coord(length.0),
                                            gecko_filter),

                HueRotate(angle) => fill_filter(NS_STYLE_FILTER_HUE_ROTATE,
                                                CoordDataValue::from(angle),
                                                gecko_filter),

                DropShadow(shadow) => {
                    gecko_filter.mType = NS_STYLE_FILTER_DROP_SHADOW;

                    fn init_shadow(filter: &mut nsStyleFilter) -> &mut nsCSSShadowArray {
                        unsafe {
                            let ref mut union = filter.__bindgen_anon_1;
                            let mut shadow_array: &mut *mut nsCSSShadowArray = union.mDropShadow.as_mut();
                            *shadow_array = Gecko_NewCSSShadowArray(1);

                            &mut **shadow_array
                        }
                    }

                    let mut gecko_shadow = init_shadow(gecko_filter);
                    gecko_shadow.mArray[0].set_from_simple_shadow(shadow);
                },
                Url(ref url) => {
                    unsafe {
                        bindings::Gecko_nsStyleFilter_SetURLValue(gecko_filter, url.for_ffi());
                    }
                },
            }
        }
    }

    pub fn copy_filter_from(&mut self, other: &Self) {
        unsafe {
            Gecko_CopyFiltersFrom(&other.gecko as *const _ as *mut _, &mut self.gecko);
        }
    }

    pub fn clone_filter(&self) -> longhands::filter::computed_value::T {
        use values::generics::effects::Filter;
        use values::specified::url::SpecifiedUrl;
        use gecko_bindings::structs::NS_STYLE_FILTER_BLUR;
        use gecko_bindings::structs::NS_STYLE_FILTER_BRIGHTNESS;
        use gecko_bindings::structs::NS_STYLE_FILTER_CONTRAST;
        use gecko_bindings::structs::NS_STYLE_FILTER_GRAYSCALE;
        use gecko_bindings::structs::NS_STYLE_FILTER_INVERT;
        use gecko_bindings::structs::NS_STYLE_FILTER_OPACITY;
        use gecko_bindings::structs::NS_STYLE_FILTER_SATURATE;
        use gecko_bindings::structs::NS_STYLE_FILTER_SEPIA;
        use gecko_bindings::structs::NS_STYLE_FILTER_HUE_ROTATE;
        use gecko_bindings::structs::NS_STYLE_FILTER_DROP_SHADOW;
        use gecko_bindings::structs::NS_STYLE_FILTER_URL;

        let mut filters = Vec::new();
        for filter in self.gecko.mFilters.iter(){
            match filter.mType {
                % for func in FILTER_FUNCTIONS:
                NS_STYLE_FILTER_${func.upper()} => {
                    filters.push(Filter::${func}(
                        GeckoStyleCoordConvertible::from_gecko_style_coord(
                            &filter.mFilterParameter).unwrap()));
                },
                % endfor
                NS_STYLE_FILTER_BLUR => {
                    filters.push(Filter::Blur(Au::from_gecko_style_coord(
                        &filter.mFilterParameter).unwrap()));
                },
                NS_STYLE_FILTER_HUE_ROTATE => {
                    filters.push(Filter::HueRotate(
                        GeckoStyleCoordConvertible::from_gecko_style_coord(
                            &filter.mFilterParameter).unwrap()));
                },
                NS_STYLE_FILTER_DROP_SHADOW => {
                    filters.push(unsafe {
                        Filter::DropShadow(
                            (**filter.__bindgen_anon_1.mDropShadow.as_ref()).mArray[0].to_simple_shadow(),
                        )
                    });
                },
                NS_STYLE_FILTER_URL => {
                    filters.push(unsafe {
                        Filter::Url(
                            SpecifiedUrl::from_url_value_data(&(**filter.__bindgen_anon_1.mURL.as_ref())._base).unwrap()
                        )
                    });
                }
                _ => {},
            }
        }
        longhands::filter::computed_value::T(filters)
    }

</%self:impl_trait>

<%self:impl_trait style_struct_name="InheritedBox"
                  skip_longhands="image-orientation">
    // FIXME: Gecko uses a tricky way to store computed value of image-orientation
    //        within an u8. We could inline following glue codes by implementing all
    //        those tricky parts for Servo as well. But, it's not done yet just for
    //        convenience.
    pub fn set_image_orientation(&mut self, v: longhands::image_orientation::computed_value::T) {
        use properties::longhands::image_orientation::computed_value::T;
        match v {
            T::FromImage => {
                unsafe {
                    bindings::Gecko_SetImageOrientationAsFromImage(&mut self.gecko);
                }
            },
            T::AngleWithFlipped(ref angle, flipped) => {
                unsafe {
                    bindings::Gecko_SetImageOrientation(&mut self.gecko, angle.radians() as f64, flipped);
                }
            }
        }
    }

    pub fn copy_image_orientation_from(&mut self, other: &Self) {
        unsafe {
            bindings::Gecko_CopyImageOrientationFrom(&mut self.gecko, &other.gecko);
        }
    }
</%self:impl_trait>

<%self:impl_trait style_struct_name="InheritedTable"
                  skip_longhands="border-spacing">

    pub fn set_border_spacing(&mut self, v: longhands::border_spacing::computed_value::T) {
        self.gecko.mBorderSpacingCol = v.horizontal.0;
        self.gecko.mBorderSpacingRow = v.vertical.0;
    }

    pub fn copy_border_spacing_from(&mut self, other: &Self) {
        self.gecko.mBorderSpacingCol = other.gecko.mBorderSpacingCol;
        self.gecko.mBorderSpacingRow = other.gecko.mBorderSpacingRow;
    }

    pub fn clone_border_spacing(&self) -> longhands::border_spacing::computed_value::T {
        longhands::border_spacing::computed_value::T {
            horizontal: Au(self.gecko.mBorderSpacingCol),
            vertical: Au(self.gecko.mBorderSpacingRow)
        }
    }
</%self:impl_trait>


<%self:impl_trait style_struct_name="InheritedText"
                  skip_longhands="text-align text-emphasis-style text-shadow line-height letter-spacing word-spacing
                                  -webkit-text-stroke-width text-emphasis-position -moz-tab-size">

    <% text_align_keyword = Keyword("text-align",
                                    "start end left right center justify -moz-center -moz-left -moz-right char",
                                    gecko_strip_moz_prefix=False) %>
    ${impl_keyword('text_align', 'mTextAlign', text_align_keyword, need_clone=False)}
    ${impl_keyword_clone('text_align', 'mTextAlign', text_align_keyword)}

    pub fn set_text_shadow<I>(&mut self, v: I)
        where I: IntoIterator<Item = SimpleShadow>,
              I::IntoIter: ExactSizeIterator
    {
        let v = v.into_iter();
        self.gecko.mTextShadow.replace_with_new(v.len() as u32);
        for (servo, gecko_shadow) in v.zip(self.gecko.mTextShadow.iter_mut()) {
            gecko_shadow.set_from_simple_shadow(servo);
        }
    }

    pub fn copy_text_shadow_from(&mut self, other: &Self) {
        self.gecko.mTextShadow.copy_from(&other.gecko.mTextShadow);
    }

    pub fn clone_text_shadow(&self) -> longhands::text_shadow::computed_value::T {
        let buf = self.gecko.mTextShadow.iter().map(|v| v.to_simple_shadow()).collect();
        longhands::text_shadow::computed_value::T(buf)
    }

    pub fn set_line_height(&mut self, v: longhands::line_height::computed_value::T) {
        use values::generics::text::LineHeight;
        // FIXME: Align binary representations and ditch |match| for cast + static_asserts
        let en = match v {
            LineHeight::Normal => CoordDataValue::Normal,
            LineHeight::Length(val) => CoordDataValue::Coord(val.0),
            LineHeight::Number(val) => CoordDataValue::Factor(val),
            LineHeight::MozBlockHeight =>
                    CoordDataValue::Enumerated(structs::NS_STYLE_LINE_HEIGHT_BLOCK_HEIGHT),
        };
        self.gecko.mLineHeight.set_value(en);
    }

    pub fn clone_line_height(&self) -> longhands::line_height::computed_value::T {
        use values::generics::text::LineHeight;
        return match self.gecko.mLineHeight.as_value() {
            CoordDataValue::Normal => LineHeight::Normal,
            CoordDataValue::Coord(coord) => LineHeight::Length(Au(coord)),
            CoordDataValue::Factor(n) => LineHeight::Number(n),
            CoordDataValue::Enumerated(val) if val == structs::NS_STYLE_LINE_HEIGHT_BLOCK_HEIGHT =>
                LineHeight::MozBlockHeight,
            _ => panic!("this should not happen"),
        }
    }

    <%call expr="impl_coord_copy('line_height', 'mLineHeight')"></%call>

    pub fn set_letter_spacing(&mut self, v: longhands::letter_spacing::computed_value::T) {
        use values::generics::text::Spacing;
        match v {
            Spacing::Value(value) => self.gecko.mLetterSpacing.set(value),
            Spacing::Normal => self.gecko.mLetterSpacing.set_value(CoordDataValue::Normal)
        }
    }

    pub fn clone_letter_spacing(&self) -> longhands::letter_spacing::computed_value::T {
        use values::generics::text::Spacing;
        debug_assert!(
            matches!(self.gecko.mLetterSpacing.as_value(),
                     CoordDataValue::Normal |
                     CoordDataValue::Coord(_)),
            "Unexpected computed value for letter-spacing");
        Au::from_gecko_style_coord(&self.gecko.mLetterSpacing).map_or(Spacing::Normal, Spacing::Value)
    }

    <%call expr="impl_coord_copy('letter_spacing', 'mLetterSpacing')"></%call>

    pub fn set_word_spacing(&mut self, v: longhands::word_spacing::computed_value::T) {
        use values::generics::text::Spacing;
        match v {
            Spacing::Value(lop) => self.gecko.mWordSpacing.set(lop),
            // https://drafts.csswg.org/css-text-3/#valdef-word-spacing-normal
            Spacing::Normal => self.gecko.mWordSpacing.set_value(CoordDataValue::Coord(0)),
        }
    }

    pub fn clone_word_spacing(&self) -> longhands::word_spacing::computed_value::T {
        use values::computed::LengthOrPercentage;
        use values::generics::text::Spacing;
        debug_assert!(
            matches!(self.gecko.mWordSpacing.as_value(),
                     CoordDataValue::Normal |
                     CoordDataValue::Coord(_) |
                     CoordDataValue::Percent(_) |
                     CoordDataValue::Calc(_)),
            "Unexpected computed value for word-spacing");
        LengthOrPercentage::from_gecko_style_coord(&self.gecko.mWordSpacing).map_or(Spacing::Normal, Spacing::Value)
    }

    <%call expr="impl_coord_copy('word_spacing', 'mWordSpacing')"></%call>

    fn clear_text_emphasis_style_if_string(&mut self) {
        use nsstring::nsString;
        if self.gecko.mTextEmphasisStyle == structs::NS_STYLE_TEXT_EMPHASIS_STYLE_STRING as u8 {
            self.gecko.mTextEmphasisStyleString.assign(&nsString::new());
            self.gecko.mTextEmphasisStyle = structs::NS_STYLE_TEXT_EMPHASIS_STYLE_NONE as u8;
        }
    }

    ${impl_simple_type_with_conversion("text_emphasis_position")}

    pub fn set_text_emphasis_style(&mut self, v: longhands::text_emphasis_style::computed_value::T) {
        use properties::longhands::text_emphasis_style::computed_value::T;
        use properties::longhands::text_emphasis_style::ShapeKeyword;

        self.clear_text_emphasis_style_if_string();
        let (te, s) = match v {
            T::None => (structs::NS_STYLE_TEXT_EMPHASIS_STYLE_NONE, ""),
            T::Keyword(ref keyword) => {
                let fill = if keyword.fill {
                    structs::NS_STYLE_TEXT_EMPHASIS_STYLE_FILLED
                } else {
                    structs::NS_STYLE_TEXT_EMPHASIS_STYLE_OPEN
                };
                let shape = match keyword.shape {
                    ShapeKeyword::Dot => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_DOT,
                    ShapeKeyword::Circle => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_CIRCLE,
                    ShapeKeyword::DoubleCircle => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_DOUBLE_CIRCLE,
                    ShapeKeyword::Triangle => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_TRIANGLE,
                    ShapeKeyword::Sesame => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_SESAME,
                };

                (shape | fill, keyword.shape.char(keyword.fill))
            },
            T::String(ref s) => {
                (structs::NS_STYLE_TEXT_EMPHASIS_STYLE_STRING, &**s)
            },
        };
        self.gecko.mTextEmphasisStyleString.assign_utf8(s);
        self.gecko.mTextEmphasisStyle = te as u8;
    }

    pub fn copy_text_emphasis_style_from(&mut self, other: &Self) {
        self.clear_text_emphasis_style_if_string();
        if other.gecko.mTextEmphasisStyle == structs::NS_STYLE_TEXT_EMPHASIS_STYLE_STRING as u8 {
            self.gecko.mTextEmphasisStyleString
                      .assign(&*other.gecko.mTextEmphasisStyleString)
        }
        self.gecko.mTextEmphasisStyle = other.gecko.mTextEmphasisStyle;
    }

    pub fn clone_text_emphasis_style(&self) -> longhands::text_emphasis_style::computed_value::T {
        use properties::longhands::text_emphasis_style::computed_value::{T, KeywordValue};
        use properties::longhands::text_emphasis_style::ShapeKeyword;

        if self.gecko.mTextEmphasisStyle == structs::NS_STYLE_TEXT_EMPHASIS_STYLE_NONE as u8 {
            return T::None;
        } else if self.gecko.mTextEmphasisStyle == structs::NS_STYLE_TEXT_EMPHASIS_STYLE_STRING as u8 {
            return T::String(self.gecko.mTextEmphasisStyleString.to_string());
        }

        let fill = self.gecko.mTextEmphasisStyle & structs::NS_STYLE_TEXT_EMPHASIS_STYLE_OPEN as u8 == 0;
        let shape =
            match self.gecko.mTextEmphasisStyle as u32 & !structs::NS_STYLE_TEXT_EMPHASIS_STYLE_OPEN {
                structs::NS_STYLE_TEXT_EMPHASIS_STYLE_DOT => ShapeKeyword::Dot,
                structs::NS_STYLE_TEXT_EMPHASIS_STYLE_CIRCLE => ShapeKeyword::Circle,
                structs::NS_STYLE_TEXT_EMPHASIS_STYLE_DOUBLE_CIRCLE => ShapeKeyword::DoubleCircle,
                structs::NS_STYLE_TEXT_EMPHASIS_STYLE_TRIANGLE => ShapeKeyword::Triangle,
                structs::NS_STYLE_TEXT_EMPHASIS_STYLE_SESAME => ShapeKeyword::Sesame,
                x => panic!("Unexpected value in style struct for text-emphasis-style property: {:?}", x)
            };

        T::Keyword(KeywordValue {
            fill: fill,
            shape: shape
        })
    }

    <%call expr="impl_app_units('_webkit_text_stroke_width', 'mWebkitTextStrokeWidth', need_clone=True)"></%call>

    #[allow(non_snake_case)]
    pub fn set__moz_tab_size(&mut self, v: longhands::_moz_tab_size::computed_value::T) {
        use values::Either;

        match v {
            Either::Second(number) => {
                self.gecko.mTabSize.set_value(CoordDataValue::Factor(number));
            }
            Either::First(au) => {
                self.gecko.mTabSize.set(au);
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn clone__moz_tab_size(&self) -> longhands::_moz_tab_size::computed_value::T {
        use values::Either;

        match self.gecko.mTabSize.as_value() {
            CoordDataValue::Coord(coord) => Either::First(Au(coord)),
            CoordDataValue::Factor(number) => Either::Second(number),
            _ => unreachable!(),
        }
    }

    <%call expr="impl_coord_copy('_moz_tab_size', 'mTabSize')"></%call>
</%self:impl_trait>

<%self:impl_trait style_struct_name="Text"
                  skip_longhands="text-decoration-line text-overflow initial-letter"
                  skip_additionals="*">

    ${impl_simple_type_with_conversion("text_decoration_line")}

    fn clear_overflow_sides_if_string(&mut self) {
        use gecko_bindings::structs::nsStyleTextOverflowSide;
        use nsstring::nsString;
        fn clear_if_string(side: &mut nsStyleTextOverflowSide) {
            if side.mType == structs::NS_STYLE_TEXT_OVERFLOW_STRING as u8 {
                side.mString.assign(&nsString::new());
                side.mType = structs::NS_STYLE_TEXT_OVERFLOW_CLIP as u8;
            }
        }
        clear_if_string(&mut self.gecko.mTextOverflow.mLeft);
        clear_if_string(&mut self.gecko.mTextOverflow.mRight);
    }
    pub fn set_text_overflow(&mut self, v: longhands::text_overflow::computed_value::T) {
        use gecko_bindings::structs::nsStyleTextOverflowSide;
        use properties::longhands::text_overflow::Side;

        fn set(side: &mut nsStyleTextOverflowSide, value: &Side) {
            let ty = match *value {
                Side::Clip => structs::NS_STYLE_TEXT_OVERFLOW_CLIP,
                Side::Ellipsis => structs::NS_STYLE_TEXT_OVERFLOW_ELLIPSIS,
                Side::String(ref s) => {
                    side.mString.assign_utf8(s);
                    structs::NS_STYLE_TEXT_OVERFLOW_STRING
                }
            };
            side.mType = ty as u8;
        }

        self.clear_overflow_sides_if_string();
        self.gecko.mTextOverflow.mLogicalDirections = v.sides_are_logical;

        set(&mut self.gecko.mTextOverflow.mLeft, &v.first);
        set(&mut self.gecko.mTextOverflow.mRight, &v.second);
    }

    pub fn copy_text_overflow_from(&mut self, other: &Self) {
        use gecko_bindings::structs::nsStyleTextOverflowSide;
        fn set(side: &mut nsStyleTextOverflowSide, other: &nsStyleTextOverflowSide) {
            if other.mType == structs::NS_STYLE_TEXT_OVERFLOW_STRING as u8 {
                side.mString.assign(&*other.mString)
            }
            side.mType = other.mType
        }
        self.clear_overflow_sides_if_string();
        set(&mut self.gecko.mTextOverflow.mLeft, &other.gecko.mTextOverflow.mLeft);
        set(&mut self.gecko.mTextOverflow.mRight, &other.gecko.mTextOverflow.mRight);
        self.gecko.mTextOverflow.mLogicalDirections = other.gecko.mTextOverflow.mLogicalDirections;
    }

    pub fn clone_text_overflow(&self) -> longhands::text_overflow::computed_value::T {
        use gecko_bindings::structs::nsStyleTextOverflowSide;
        use properties::longhands::text_overflow::Side;

        fn to_servo(side: &nsStyleTextOverflowSide) -> Side {
            match side.mType as u32 {
                structs::NS_STYLE_TEXT_OVERFLOW_CLIP => Side::Clip,
                structs::NS_STYLE_TEXT_OVERFLOW_ELLIPSIS => Side::Ellipsis,
                structs::NS_STYLE_TEXT_OVERFLOW_STRING => Side::String(side.mString.to_string().into_boxed_str()),
                x => panic!("Found unexpected value in style struct for text_overflow property: {:?}", x),
            }
        }

        longhands::text_overflow::computed_value::T {
            first: to_servo(&self.gecko.mTextOverflow.mLeft),
            second: to_servo(&self.gecko.mTextOverflow.mRight),
            sides_are_logical: self.gecko.mTextOverflow.mLogicalDirections
        }
    }

    pub fn set_initial_letter(&mut self, v: longhands::initial_letter::computed_value::T) {
        use values::generics::text::InitialLetter;
        match v {
            InitialLetter::Normal => {
                self.gecko.mInitialLetterSize = 0.;
                self.gecko.mInitialLetterSink = 0;
            },
            InitialLetter::Specified(size, sink) => {
                self.gecko.mInitialLetterSize = size;
                if let Some(sink) = sink {
                    self.gecko.mInitialLetterSink = sink;
                } else {
                    self.gecko.mInitialLetterSink = size.floor() as i32;
                }
            }
        }
    }

    pub fn copy_initial_letter_from(&mut self, other: &Self) {
        self.gecko.mInitialLetterSize = other.gecko.mInitialLetterSize;
        self.gecko.mInitialLetterSink = other.gecko.mInitialLetterSink;
    }

    pub fn clone_initial_letter(&self) -> longhands::initial_letter::computed_value::T {
        use values::generics::text::InitialLetter;

        if self.gecko.mInitialLetterSize == 0. && self.gecko.mInitialLetterSink == 0 {
            InitialLetter::Normal
        } else if self.gecko.mInitialLetterSize.floor() as i32 == self.gecko.mInitialLetterSink {
            InitialLetter::Specified(self.gecko.mInitialLetterSize, None)
        } else {
            InitialLetter::Specified(self.gecko.mInitialLetterSize, Some(self.gecko.mInitialLetterSink))
        }
    }

    #[inline]
    pub fn has_underline(&self) -> bool {
        (self.gecko.mTextDecorationLine & (structs::NS_STYLE_TEXT_DECORATION_LINE_UNDERLINE as u8)) != 0
    }

    #[inline]
    pub fn has_overline(&self) -> bool {
        (self.gecko.mTextDecorationLine & (structs::NS_STYLE_TEXT_DECORATION_LINE_OVERLINE as u8)) != 0
    }

    #[inline]
    pub fn has_line_through(&self) -> bool {
        (self.gecko.mTextDecorationLine & (structs::NS_STYLE_TEXT_DECORATION_LINE_LINE_THROUGH as u8)) != 0
    }
</%self:impl_trait>

<%def name="impl_shape_source(ident, gecko_ffi_name)">
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        use gecko_bindings::bindings::{Gecko_NewBasicShape, Gecko_DestroyShapeSource};
        use gecko_bindings::structs::{StyleBasicShape, StyleBasicShapeType, StyleShapeSourceType};
        use gecko_bindings::structs::{StyleFillRule, StyleGeometryBox, StyleShapeSource};
        use gecko::conversions::basic_shape::set_corners_from_radius;
        use gecko::values::GeckoStyleCoordConvertible;
        use values::generics::basic_shape::{BasicShape, FillRule, ShapeSource};
        let ref mut ${ident} = self.gecko.${gecko_ffi_name};
        // clean up existing struct
        unsafe { Gecko_DestroyShapeSource(${ident}) };

        ${ident}.mType = StyleShapeSourceType::None;

        match v {
            ShapeSource::Url(ref url) => {
                unsafe {
                    bindings::Gecko_StyleShapeSource_SetURLValue(${ident}, url.for_ffi())
                }
            }
            ShapeSource::None => {} // don't change the type
            ShapeSource::Box(reference) => {
                ${ident}.mReferenceBox = reference.into();
                ${ident}.mType = StyleShapeSourceType::Box;
            }
            ShapeSource::Shape(servo_shape, maybe_box) => {
                ${ident}.mReferenceBox = maybe_box.map(Into::into)
                                                   .unwrap_or(StyleGeometryBox::NoBox);
                ${ident}.mType = StyleShapeSourceType::Shape;

                fn init_shape(${ident}: &mut StyleShapeSource, ty: StyleBasicShapeType) -> &mut StyleBasicShape {
                    unsafe {
                        // We have to be very careful to avoid a copy here!
                        let ref mut union = ${ident}.__bindgen_anon_1;
                        let mut shape: &mut *mut StyleBasicShape = union.mBasicShape.as_mut();
                        *shape = Gecko_NewBasicShape(ty);
                        &mut **shape
                    }
                }
                match servo_shape {
                    BasicShape::Inset(inset) => {
                        let mut shape = init_shape(${ident}, StyleBasicShapeType::Inset);
                        unsafe { shape.mCoordinates.set_len(4) };

                        // set_len() can't call constructors, so the coordinates
                        // can contain any value. set_value() attempts to free
                        // allocated coordinates, so we don't want to feed it
                        // garbage values which it may misinterpret.
                        // Instead, we use leaky_set_value to blindly overwrite
                        // the garbage data without
                        // attempting to clean up.
                        shape.mCoordinates[0].leaky_set_null();
                        inset.rect.0.to_gecko_style_coord(&mut shape.mCoordinates[0]);
                        shape.mCoordinates[1].leaky_set_null();
                        inset.rect.1.to_gecko_style_coord(&mut shape.mCoordinates[1]);
                        shape.mCoordinates[2].leaky_set_null();
                        inset.rect.2.to_gecko_style_coord(&mut shape.mCoordinates[2]);
                        shape.mCoordinates[3].leaky_set_null();
                        inset.rect.3.to_gecko_style_coord(&mut shape.mCoordinates[3]);

                        set_corners_from_radius(inset.round, &mut shape.mRadius);
                    }
                    BasicShape::Circle(circ) => {
                        let mut shape = init_shape(${ident}, StyleBasicShapeType::Circle);
                        unsafe { shape.mCoordinates.set_len(1) };
                        shape.mCoordinates[0].leaky_set_null();
                        circ.radius.to_gecko_style_coord(&mut shape.mCoordinates[0]);

                        shape.mPosition = circ.position.into();
                    }
                    BasicShape::Ellipse(el) => {
                        let mut shape = init_shape(${ident}, StyleBasicShapeType::Ellipse);
                        unsafe { shape.mCoordinates.set_len(2) };
                        shape.mCoordinates[0].leaky_set_null();
                        el.semiaxis_x.to_gecko_style_coord(&mut shape.mCoordinates[0]);
                        shape.mCoordinates[1].leaky_set_null();
                        el.semiaxis_y.to_gecko_style_coord(&mut shape.mCoordinates[1]);

                        shape.mPosition = el.position.into();
                    }
                    BasicShape::Polygon(poly) => {
                        let mut shape = init_shape(${ident}, StyleBasicShapeType::Polygon);
                        unsafe {
                            shape.mCoordinates.set_len(poly.coordinates.len() as u32 * 2);
                        }
                        for (i, coord) in poly.coordinates.iter().enumerate() {
                            shape.mCoordinates[2 * i].leaky_set_null();
                            shape.mCoordinates[2 * i + 1].leaky_set_null();
                            coord.0.to_gecko_style_coord(&mut shape.mCoordinates[2 * i]);
                            coord.1.to_gecko_style_coord(&mut shape.mCoordinates[2 * i + 1]);
                        }
                        shape.mFillRule = if poly.fill == FillRule::EvenOdd {
                            StyleFillRule::Evenodd
                        } else {
                            StyleFillRule::Nonzero
                        };
                    }
                }
            }
        }

    }

    pub fn copy_${ident}_from(&mut self, other: &Self) {
        use gecko_bindings::bindings::Gecko_CopyShapeSourceFrom;
        unsafe {
            Gecko_CopyShapeSourceFrom(&mut self.gecko.${gecko_ffi_name}, &other.gecko.${gecko_ffi_name});
        }
    }
</%def>

<% skip_svg_longhands = """
mask-mode mask-repeat mask-clip mask-origin mask-composite mask-position-x mask-position-y mask-size mask-image
clip-path
"""
%>
<%self:impl_trait style_struct_name="SVG"
                  skip_longhands="${skip_svg_longhands}"
                  skip_additionals="*">

    <% impl_common_image_layer_properties("mask") %>
    <% impl_simple_image_array_property("mode", "mask", "mMask", "mMaskMode", "SVG") %>
    <% impl_simple_image_array_property("composite", "mask", "mMask", "mComposite", "SVG") %>
    <% impl_shape_source("clip_path", "mClipPath") %>
</%self:impl_trait>

<%self:impl_trait style_struct_name="InheritedSVG"
                  skip_longhands="paint-order stroke-dasharray stroke-dashoffset stroke-width -moz-context-properties"
                  skip_additionals="*">
    pub fn set_paint_order(&mut self, v: longhands::paint_order::computed_value::T) {
        use self::longhands::paint_order;

        if v.0 == 0 {
            self.gecko.mPaintOrder = structs::NS_STYLE_PAINT_ORDER_NORMAL as u8;
        } else {
            let mut order = 0;

            for pos in 0..3 {
                let geckoval = match v.bits_at(pos) {
                    paint_order::FILL => structs::NS_STYLE_PAINT_ORDER_FILL as u8,
                    paint_order::STROKE => structs::NS_STYLE_PAINT_ORDER_STROKE as u8,
                    paint_order::MARKERS => structs::NS_STYLE_PAINT_ORDER_MARKERS as u8,
                    _ => unreachable!(),
                };
                order |= geckoval << (pos * structs::NS_STYLE_PAINT_ORDER_BITWIDTH as u8);
            }

            self.gecko.mPaintOrder = order;
        }
    }

    ${impl_simple_copy('paint_order', 'mPaintOrder')}

    pub fn set_stroke_dasharray<I>(&mut self, v: I)
        where I: IntoIterator<Item = longhands::stroke_dasharray::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        let v = v.into_iter();
        unsafe {
            bindings::Gecko_nsStyleSVG_SetDashArrayLength(&mut self.gecko, v.len() as u32);
        }

        for (mut gecko, servo) in self.gecko.mStrokeDasharray.iter_mut().zip(v) {
            match servo {
                Either::First(number) => gecko.set_value(CoordDataValue::Factor(number)),
                Either::Second(lop) => gecko.set(lop),
            }
        }
    }

    pub fn copy_stroke_dasharray_from(&mut self, other: &Self) {
        unsafe {
            bindings::Gecko_nsStyleSVG_CopyDashArray(&mut self.gecko, &other.gecko);
        }
    }

    pub fn clone_stroke_dasharray(&self) -> longhands::stroke_dasharray::computed_value::T {
        use values::computed::LengthOrPercentage;

        let mut vec = vec![];
        for gecko in self.gecko.mStrokeDasharray.iter() {
            match gecko.as_value() {
                CoordDataValue::Factor(number) => vec.push(Either::First(number)),
                CoordDataValue::Coord(coord) =>
                    vec.push(Either::Second(LengthOrPercentage::Length(Au(coord)))),
                CoordDataValue::Percent(p) =>
                    vec.push(Either::Second(LengthOrPercentage::Percentage(Percentage(p)))),
                CoordDataValue::Calc(calc) =>
                    vec.push(Either::Second(LengthOrPercentage::Calc(calc.into()))),
                _ => unreachable!(),
            }
        }
        longhands::stroke_dasharray::computed_value::T(vec)
    }

    pub fn set_stroke_dashoffset(&mut self, v: longhands::stroke_dashoffset::computed_value::T) {
        match v {
            Either::First(number) => self.gecko.mStrokeDashoffset.set_value(CoordDataValue::Factor(number)),
            Either::Second(lop) => self.gecko.mStrokeDashoffset.set(lop),
        }
    }

    ${impl_coord_copy('stroke_dashoffset', 'mStrokeDashoffset')}

    pub fn clone_stroke_dashoffset(&self) -> longhands::stroke_dashoffset::computed_value::T {
        use values::computed::LengthOrPercentage;
        match self.gecko.mStrokeDashoffset.as_value() {
            CoordDataValue::Factor(number) => Either::First(number),
            CoordDataValue::Coord(coord) => Either::Second(LengthOrPercentage::Length(Au(coord))),
            CoordDataValue::Percent(p) => Either::Second(LengthOrPercentage::Percentage(Percentage(p))),
            CoordDataValue::Calc(calc) => Either::Second(LengthOrPercentage::Calc(calc.into())),
            _ => unreachable!(),
        }
    }

    pub fn set_stroke_width(&mut self, v: longhands::stroke_width::computed_value::T) {
        match v {
            Either::First(number) => self.gecko.mStrokeWidth.set_value(CoordDataValue::Factor(number)),
            Either::Second(lop) => self.gecko.mStrokeWidth.set(lop),
        }
    }

    ${impl_coord_copy('stroke_width', 'mStrokeWidth')}

    pub fn clone_stroke_width(&self) -> longhands::stroke_width::computed_value::T {
        use values::computed::LengthOrPercentage;
        match self.gecko.mStrokeWidth.as_value() {
            CoordDataValue::Factor(number) => Either::First(number),
            CoordDataValue::Coord(coord) => Either::Second(LengthOrPercentage::Length(Au(coord))),
            CoordDataValue::Percent(p) => Either::Second(LengthOrPercentage::Percentage(Percentage(p))),
            CoordDataValue::Calc(calc) => Either::Second(LengthOrPercentage::Calc(calc.into())),
            _ => unreachable!(),
        }
    }

    #[allow(non_snake_case)]
    pub fn set__moz_context_properties<I>(&mut self, v: I)
        where I: IntoIterator<Item = longhands::_moz_context_properties::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        let v = v.into_iter();
        unsafe {
            bindings::Gecko_nsStyleSVG_SetContextPropertiesLength(&mut self.gecko, v.len() as u32);
        }

        self.gecko.mContextPropsBits = 0;
        for (mut gecko, servo) in self.gecko.mContextProps.iter_mut().zip(v) {
            if servo.0 == atom!("fill") {
                self.gecko.mContextPropsBits |= structs::NS_STYLE_CONTEXT_PROPERTY_FILL as u8;
            } else if servo.0 == atom!("stroke") {
                self.gecko.mContextPropsBits |= structs::NS_STYLE_CONTEXT_PROPERTY_STROKE as u8;
            } else if servo.0 == atom!("fill-opacity") {
                self.gecko.mContextPropsBits |= structs::NS_STYLE_CONTEXT_PROPERTY_FILL_OPACITY as u8;
            } else if servo.0 == atom!("stroke-opacity") {
                self.gecko.mContextPropsBits |= structs::NS_STYLE_CONTEXT_PROPERTY_STROKE_OPACITY as u8;
            }
            unsafe { gecko.set_raw_from_addrefed::<structs::nsIAtom>(servo.0.into_addrefed()) }
        }
    }

    #[allow(non_snake_case)]
    pub fn copy__moz_context_properties_from(&mut self, other: &Self) {
        unsafe {
            bindings::Gecko_nsStyleSVG_CopyContextProperties(&mut self.gecko, &other.gecko);
        }
    }
</%self:impl_trait>

<%self:impl_trait style_struct_name="Color"
                  skip_longhands="*">
    pub fn set_color(&mut self, v: longhands::color::computed_value::T) {
        let result = convert_rgba_to_nscolor(&v);
        ${set_gecko_property("mColor", "result")}
    }

    <%call expr="impl_simple_copy('color', 'mColor')"></%call>

    pub fn clone_color(&self) -> longhands::color::computed_value::T {
        let color = ${get_gecko_property("mColor")} as u32;
        convert_nscolor_to_rgba(color)
    }
</%self:impl_trait>

<%self:impl_trait style_struct_name="Pointing"
                  skip_longhands="cursor caret-color">
    pub fn set_cursor(&mut self, v: longhands::cursor::computed_value::T) {
        use properties::longhands::cursor::computed_value::Keyword;
        use style_traits::cursor::Cursor;

        self.gecko.mCursor = match v.keyword {
            Keyword::Auto => structs::NS_STYLE_CURSOR_AUTO,
            Keyword::Cursor(cursor) => match cursor {
                Cursor::None => structs::NS_STYLE_CURSOR_NONE,
                Cursor::Default => structs::NS_STYLE_CURSOR_DEFAULT,
                Cursor::Pointer => structs::NS_STYLE_CURSOR_POINTER,
                Cursor::ContextMenu => structs::NS_STYLE_CURSOR_CONTEXT_MENU,
                Cursor::Help => structs::NS_STYLE_CURSOR_HELP,
                Cursor::Progress => structs::NS_STYLE_CURSOR_DEFAULT, // Gecko doesn't support "progress" yet
                Cursor::Wait => structs::NS_STYLE_CURSOR_WAIT,
                Cursor::Cell => structs::NS_STYLE_CURSOR_CELL,
                Cursor::Crosshair => structs::NS_STYLE_CURSOR_CROSSHAIR,
                Cursor::Text => structs::NS_STYLE_CURSOR_TEXT,
                Cursor::VerticalText => structs::NS_STYLE_CURSOR_VERTICAL_TEXT,
                Cursor::Alias => structs::NS_STYLE_CURSOR_ALIAS,
                Cursor::Copy => structs::NS_STYLE_CURSOR_COPY,
                Cursor::Move => structs::NS_STYLE_CURSOR_MOVE,
                Cursor::NoDrop => structs::NS_STYLE_CURSOR_NO_DROP,
                Cursor::NotAllowed => structs::NS_STYLE_CURSOR_NOT_ALLOWED,
                Cursor::Grab => structs::NS_STYLE_CURSOR_GRAB,
                Cursor::Grabbing => structs::NS_STYLE_CURSOR_GRABBING,
                Cursor::EResize => structs::NS_STYLE_CURSOR_E_RESIZE,
                Cursor::NResize => structs::NS_STYLE_CURSOR_N_RESIZE,
                Cursor::NeResize => structs::NS_STYLE_CURSOR_NE_RESIZE,
                Cursor::NwResize => structs::NS_STYLE_CURSOR_NW_RESIZE,
                Cursor::SResize => structs::NS_STYLE_CURSOR_S_RESIZE,
                Cursor::SeResize => structs::NS_STYLE_CURSOR_SE_RESIZE,
                Cursor::SwResize => structs::NS_STYLE_CURSOR_SW_RESIZE,
                Cursor::WResize => structs::NS_STYLE_CURSOR_W_RESIZE,
                Cursor::EwResize => structs::NS_STYLE_CURSOR_EW_RESIZE,
                Cursor::NsResize => structs::NS_STYLE_CURSOR_NS_RESIZE,
                Cursor::NeswResize => structs::NS_STYLE_CURSOR_NESW_RESIZE,
                Cursor::NwseResize => structs::NS_STYLE_CURSOR_NWSE_RESIZE,
                Cursor::ColResize => structs::NS_STYLE_CURSOR_COL_RESIZE,
                Cursor::RowResize => structs::NS_STYLE_CURSOR_ROW_RESIZE,
                Cursor::AllScroll => structs::NS_STYLE_CURSOR_ALL_SCROLL,
                Cursor::ZoomIn => structs::NS_STYLE_CURSOR_ZOOM_IN,
                Cursor::ZoomOut => structs::NS_STYLE_CURSOR_ZOOM_OUT,
                // note: the following properties are gecko-only.
                Cursor::MozGrab => structs::NS_STYLE_CURSOR_GRAB,
                Cursor::MozGrabbing => structs::NS_STYLE_CURSOR_GRABBING,
                Cursor::MozZoomIn => structs::NS_STYLE_CURSOR_ZOOM_IN,
                Cursor::MozZoomOut => structs::NS_STYLE_CURSOR_ZOOM_OUT,
            }
        } as u8;

        unsafe {
            Gecko_SetCursorArrayLength(&mut self.gecko, v.images.len());
        }
        for i in 0..v.images.len() {
            unsafe {
                Gecko_SetCursorImageValue(&mut self.gecko.mCursorImages[i],
                                          v.images[i].url.clone().image_value.unwrap().get());
            }

            // We don't need to record this struct as uncacheable, like when setting
            // background-image to a url() value, since only properties in reset structs
            // are re-used from the applicable declaration cache, and the Pointing struct
            // is an inherited struct.
        }
    }

    pub fn copy_cursor_from(&mut self, other: &Self) {
        self.gecko.mCursor = other.gecko.mCursor;
        unsafe {
            Gecko_CopyCursorArrayFrom(&mut self.gecko, &other.gecko);
        }
    }

    <%call expr="impl_color('caret_color', 'mCaretColor', need_clone=True)"></%call>
</%self:impl_trait>

<%self:impl_trait style_struct_name="Column"
                  skip_longhands="column-count column-rule-width">

    #[allow(unused_unsafe)]
    pub fn set_column_count(&mut self, v: longhands::column_count::computed_value::T) {
        use gecko_bindings::structs::{NS_STYLE_COLUMN_COUNT_AUTO, nsStyleColumn_kMaxColumnCount};

        self.gecko.mColumnCount = match v {
            Either::First(number) => unsafe {
                cmp::min(number as u32, nsStyleColumn_kMaxColumnCount)
            },
            Either::Second(Auto) => NS_STYLE_COLUMN_COUNT_AUTO
        };
    }

    ${impl_simple_copy('column_count', 'mColumnCount')}

    pub fn clone_column_count(&self) -> longhands::column_count::computed_value::T {
        use gecko_bindings::structs::NS_STYLE_COLUMN_COUNT_AUTO;
        if self.gecko.mColumnCount != NS_STYLE_COLUMN_COUNT_AUTO {
            debug_assert!((self.gecko.mColumnCount as i32) >= 0 &&
                          (self.gecko.mColumnCount as i32) < i32::max_value());
            Either::First(self.gecko.mColumnCount as i32)
        } else {
            Either::Second(Auto)
        }
    }

    <% impl_app_units("column_rule_width", "mColumnRuleWidth", need_clone=True,
                      round_to_pixels=True) %>
</%self:impl_trait>

<%self:impl_trait style_struct_name="Counters"
                  skip_longhands="content counter-increment counter-reset">
    pub fn ineffective_content_property(&self) -> bool {
        self.gecko.mContents.is_empty()
    }

    pub fn set_content(&mut self, v: longhands::content::computed_value::T, device: &Device) {
        use properties::longhands::content::computed_value::T;
        use properties::longhands::content::computed_value::ContentItem;
        use values::generics::CounterStyleOrNone;
        use gecko_bindings::structs::nsStyleContentData;
        use gecko_bindings::structs::nsStyleContentType;
        use gecko_bindings::structs::nsStyleContentType::*;
        use gecko_bindings::bindings::Gecko_ClearAndResizeStyleContents;

        // Converts a string as utf16, and returns an owned, zero-terminated raw buffer.
        fn as_utf16_and_forget(s: &str) -> *mut u16 {
            use std::mem;
            let mut vec = s.encode_utf16().collect::<Vec<_>>();
            vec.push(0u16);
            let ptr = vec.as_mut_ptr();
            mem::forget(vec);
            ptr
        }

        fn set_counter_function(data: &mut nsStyleContentData,
                                content_type: nsStyleContentType,
                                name: &str, sep: &str,
                                style: CounterStyleOrNone, device: &Device) {
            debug_assert!(content_type == eStyleContentType_Counter ||
                          content_type == eStyleContentType_Counters);
            let counter_func = unsafe {
                bindings::Gecko_SetCounterFunction(data, content_type).as_mut().unwrap()
            };
            counter_func.mIdent.assign_utf8(name);
            if content_type == eStyleContentType_Counters {
                counter_func.mSeparator.assign_utf8(sep);
            }
            style.to_gecko_value(&mut counter_func.mCounterStyle, device);
        }

        match v {
            T::None |
            T::Normal => {
                // Ensure destructors run, otherwise we could leak.
                if !self.gecko.mContents.is_empty() {
                    unsafe {
                        Gecko_ClearAndResizeStyleContents(&mut self.gecko, 0);
                    }
                }
            },
            T::MozAltContent => {
                unsafe {
                    Gecko_ClearAndResizeStyleContents(&mut self.gecko, 1);
                    *self.gecko.mContents[0].mContent.mString.as_mut() = ptr::null_mut();
                }
                self.gecko.mContents[0].mType = eStyleContentType_AltContent;
            },
            T::Items(items) => {
                unsafe {
                    Gecko_ClearAndResizeStyleContents(&mut self.gecko,
                                                      items.len() as u32);
                }
                for (i, item) in items.into_iter().enumerate() {
                    // NB: Gecko compares the mString value if type is not image
                    // or URI independently of whatever gets there. In the quote
                    // cases, they set it to null, so do the same here.
                    unsafe {
                        *self.gecko.mContents[i].mContent.mString.as_mut() = ptr::null_mut();
                    }
                    match item {
                        ContentItem::String(value) => {
                            self.gecko.mContents[i].mType = eStyleContentType_String;
                            unsafe {
                                // NB: we share allocators, so doing this is fine.
                                *self.gecko.mContents[i].mContent.mString.as_mut() =
                                    as_utf16_and_forget(&value);
                            }
                        }
                        ContentItem::Attr(attr) => {
                            self.gecko.mContents[i].mType = eStyleContentType_Attr;
                            let s = if let Some((_, ns)) = attr.namespace {
                                format!("{}|{}", ns, attr.attribute)
                            } else {
                                attr.attribute.into()
                            };
                            unsafe {
                                // NB: we share allocators, so doing this is fine.
                                *self.gecko.mContents[i].mContent.mString.as_mut() =
                                    as_utf16_and_forget(&s);
                            }
                        }
                        ContentItem::OpenQuote
                            => self.gecko.mContents[i].mType = eStyleContentType_OpenQuote,
                        ContentItem::CloseQuote
                            => self.gecko.mContents[i].mType = eStyleContentType_CloseQuote,
                        ContentItem::NoOpenQuote
                            => self.gecko.mContents[i].mType = eStyleContentType_NoOpenQuote,
                        ContentItem::NoCloseQuote
                            => self.gecko.mContents[i].mType = eStyleContentType_NoCloseQuote,
                        ContentItem::Counter(name, style) => {
                            set_counter_function(&mut self.gecko.mContents[i],
                                                 eStyleContentType_Counter, &name, "", style, device);
                        }
                        ContentItem::Counters(name, sep, style) => {
                            set_counter_function(&mut self.gecko.mContents[i],
                                                 eStyleContentType_Counters, &name, &sep, style, device);
                        }
                        ContentItem::Url(ref url) => {
                            unsafe {
                                bindings::Gecko_SetContentDataImageValue(&mut self.gecko.mContents[i],
                                    url.image_value.clone().unwrap().get())
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn copy_content_from(&mut self, other: &Self) {
        use gecko_bindings::bindings::Gecko_CopyStyleContentsFrom;
        unsafe {
            Gecko_CopyStyleContentsFrom(&mut self.gecko, &other.gecko)
        }
    }

    % for counter_property in ["Increment", "Reset"]:
        pub fn set_counter_${counter_property.lower()}(&mut self, v: longhands::counter_increment::computed_value::T) {
            unsafe {
                bindings::Gecko_ClearAndResizeCounter${counter_property}s(&mut self.gecko,
                                                                      v.0.len() as u32);
                for (i, (name, value)) in v.0.into_iter().enumerate() {
                    self.gecko.m${counter_property}s[i].mCounter.assign(name.0.as_slice());
                    self.gecko.m${counter_property}s[i].mValue = value;
                }
            }
        }

        pub fn copy_counter_${counter_property.lower()}_from(&mut self, other: &Self) {
            unsafe {
                bindings::Gecko_CopyCounter${counter_property}sFrom(&mut self.gecko, &other.gecko)
            }
        }

        pub fn clone_counter_${counter_property.lower()}(&self) -> longhands::counter_increment::computed_value::T {
            use values::CustomIdent;
            use gecko_string_cache::Atom;

            longhands::counter_increment::computed_value::T(
                self.gecko.m${counter_property}s.iter().map(|ref gecko_counter| {
                    (CustomIdent(Atom::from(gecko_counter.mCounter.to_string())), gecko_counter.mValue)
                }).collect()
            )
        }
    % endfor
</%self:impl_trait>

<%self:impl_trait style_struct_name="UI" skip_longhands="-moz-force-broken-image-icon">
    ${impl_simple_type_with_conversion("_moz_force_broken_image_icon", "mForceBrokenImageIcon")}
</%self:impl_trait>

<%self:impl_trait style_struct_name="XUL"
                  skip_longhands="-moz-box-ordinal-group">
    #[allow(non_snake_case)]
    pub fn set__moz_box_ordinal_group(&mut self, v: i32) {
        self.gecko.mBoxOrdinal = v as u32;
    }

    ${impl_simple_copy("_moz_box_ordinal_group", "mBoxOrdinal")}

    #[allow(non_snake_case)]
    pub fn clone__moz_box_ordinal_group(&self) -> i32{
        self.gecko.mBoxOrdinal as i32
    }
</%self:impl_trait>

<%def name="define_ffi_struct_accessor(style_struct)">
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn Servo_GetStyle${style_struct.gecko_name}(computed_values:
        ServoComputedValuesBorrowedOrNull) -> *const ${style_struct.gecko_ffi_name} {
    ComputedValues::arc_from_borrowed(&computed_values).unwrap().get_${style_struct.name_lower}().get_gecko()
        as *const ${style_struct.gecko_ffi_name}
}
</%def>

% for style_struct in data.style_structs:
${declare_style_struct(style_struct)}
${impl_style_struct(style_struct)}
% if not style_struct.name in data.manual_style_structs:
<%self:raw_impl_trait style_struct="${style_struct}"></%self:raw_impl_trait>
% endif
${define_ffi_struct_accessor(style_struct)}
% endfor

// This is only accessed from the Gecko main thread.
static mut EMPTY_VARIABLES_STRUCT: Option<nsStyleVariables> = None;

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Servo_GetStyleVariables(_cv: ServoComputedValuesBorrowedOrNull)
                                                 -> *const nsStyleVariables {
    EMPTY_VARIABLES_STRUCT.as_ref().unwrap()
}

pub fn initialize() {
    unsafe {
        EMPTY_VARIABLES_STRUCT = Some(zeroed());
        Gecko_Construct_nsStyleVariables(EMPTY_VARIABLES_STRUCT.as_mut().unwrap());
    }
}

pub fn shutdown() {
    unsafe {
        EMPTY_VARIABLES_STRUCT.take().as_mut().map(|v| Gecko_Destroy_nsStyleVariables(v));
    }
}
