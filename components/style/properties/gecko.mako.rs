/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// `data` comes from components/style/properties.mako.rs; see build.rs for more details.

<%!
    from data import to_camel_case, to_camel_case_lower
    from data import Keyword
%>
<%namespace name="helpers" file="/helpers.mako.rs" />

use crate::Atom;
use app_units::Au;
use crate::custom_properties::CustomPropertiesMap;
use crate::gecko_bindings::bindings;
% for style_struct in data.style_structs:
use crate::gecko_bindings::structs::${style_struct.gecko_ffi_name};
use crate::gecko_bindings::bindings::Gecko_Construct_Default_${style_struct.gecko_ffi_name};
use crate::gecko_bindings::bindings::Gecko_CopyConstruct_${style_struct.gecko_ffi_name};
use crate::gecko_bindings::bindings::Gecko_Destroy_${style_struct.gecko_ffi_name};
% endfor
use crate::gecko_bindings::bindings::Gecko_CopyCounterStyle;
use crate::gecko_bindings::bindings::Gecko_CopyCursorArrayFrom;
use crate::gecko_bindings::bindings::Gecko_CopyFontFamilyFrom;
use crate::gecko_bindings::bindings::Gecko_CopyImageValueFrom;
use crate::gecko_bindings::bindings::Gecko_CopyListStyleImageFrom;
use crate::gecko_bindings::bindings::Gecko_EnsureImageLayersLength;
use crate::gecko_bindings::bindings::Gecko_SetCursorArrayLength;
use crate::gecko_bindings::bindings::Gecko_SetCursorImageValue;
use crate::gecko_bindings::bindings::Gecko_nsStyleFont_SetLang;
use crate::gecko_bindings::bindings::Gecko_nsStyleFont_CopyLangFrom;
use crate::gecko_bindings::bindings::Gecko_SetListStyleImageNone;
use crate::gecko_bindings::bindings::Gecko_SetListStyleImageImageValue;
use crate::gecko_bindings::bindings::Gecko_SetNullImageValue;
use crate::gecko_bindings::structs;
use crate::gecko_bindings::structs::nsCSSPropertyID;
use crate::gecko_bindings::structs::mozilla::PseudoStyleType;
use crate::gecko_bindings::sugar::ns_style_coord::CoordDataMut;
use crate::gecko_bindings::sugar::refptr::RefPtr;
use crate::gecko::values::round_border_to_device_pixels;
use crate::logical_geometry::WritingMode;
use crate::media_queries::Device;
use crate::properties::computed_value_flags::*;
use crate::properties::longhands;
use crate::rule_tree::StrongRuleNode;
use crate::selector_parser::PseudoElement;
use servo_arc::{Arc, RawOffsetArc};
use std::marker::PhantomData;
use std::mem::{forget, uninitialized, zeroed, ManuallyDrop};
use std::{cmp, ops, ptr};
use crate::values::{self, CustomIdent, Either, KeyframesName, None_};
use crate::values::computed::{Percentage, TransitionProperty};
use crate::values::computed::url::ComputedImageUrl;
use crate::values::computed::BorderStyle;
use crate::values::computed::font::FontSize;
use crate::values::generics::column::ColumnCount;
use crate::values::generics::image::ImageLayer;
use crate::values::generics::transform::TransformStyle;
use crate::values::generics::url::UrlOrNone;


pub mod style_structs {
    % for style_struct in data.style_structs:
    pub use super::${style_struct.gecko_struct_name} as ${style_struct.name};

    unsafe impl Send for ${style_struct.name} {}
    unsafe impl Sync for ${style_struct.name} {}
    % endfor

}

/// FIXME(emilio): This is completely duplicated with the other properties code.
pub type ComputedValuesInner = structs::ServoComputedData;

#[repr(C)]
pub struct ComputedValues(structs::mozilla::ComputedStyle);

impl ComputedValues {
    #[inline]
    pub (crate) fn as_gecko_computed_style(&self) -> &structs::ComputedStyle {
        &self.0
    }

    pub fn new(
        pseudo: Option<<&PseudoElement>,
        custom_properties: Option<Arc<CustomPropertiesMap>>,
        writing_mode: WritingMode,
        flags: ComputedValueFlags,
        rules: Option<StrongRuleNode>,
        visited_style: Option<Arc<ComputedValues>>,
        % for style_struct in data.style_structs:
        ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
        % endfor
    ) -> Arc<Self> {
        ComputedValuesInner::new(
            custom_properties,
            writing_mode,
            flags,
            rules,
            visited_style,
            % for style_struct in data.style_structs:
            ${style_struct.ident},
            % endfor
        ).to_outer(pseudo)
    }

    pub fn default_values(doc: &structs::Document) -> Arc<Self> {
        ComputedValuesInner::new(
            /* custom_properties = */ None,
            /* writing_mode = */ WritingMode::empty(), // FIXME(bz): This seems dubious
            ComputedValueFlags::empty(),
            /* rules = */ None,
            /* visited_style = */ None,
            % for style_struct in data.style_structs:
            style_structs::${style_struct.name}::default(doc),
            % endfor
        ).to_outer(None)
    }

    #[inline]
    pub fn pseudo(&self) -> Option<PseudoElement> {
        if self.0.mPseudoType == PseudoStyleType::NotPseudo {
            return None;
        }
        PseudoElement::from_pseudo_type(self.0.mPseudoType)
    }

    #[inline]
    pub fn is_first_line_style(&self) -> bool {
        self.pseudo() == Some(PseudoElement::FirstLine)
    }

    /// Returns true if the display property is changed from 'none' to others.
    pub fn is_display_property_changed_from_none(
        &self,
        old_values: Option<<&ComputedValues>
    ) -> bool {
        use crate::properties::longhands::display::computed_value::T as Display;

        old_values.map_or(false, |old| {
            let old_display_style = old.get_box().clone_display();
            let new_display_style = self.get_box().clone_display();
            old_display_style == Display::None &&
            new_display_style != Display::None
        })
    }

}

impl Drop for ComputedValues {
    fn drop(&mut self) {
        unsafe {
            bindings::Gecko_ComputedStyle_Destroy(&mut self.0);
        }
    }
}

unsafe impl Sync for ComputedValues {}
unsafe impl Send for ComputedValues {}

impl Clone for ComputedValues {
    fn clone(&self) -> Self {
        unreachable!()
    }
}

impl Clone for ComputedValuesInner {
    fn clone(&self) -> Self {
        ComputedValuesInner {
            % for style_struct in data.style_structs:
                ${style_struct.gecko_name}: self.${style_struct.gecko_name}.clone(),
            % endfor
            custom_properties: self.custom_properties.clone(),
            writing_mode: self.writing_mode.clone(),
            flags: self.flags.clone(),
            rules: self.rules.clone(),
            visited_style: self.visited_style.clone(),
        }
    }
}

impl ComputedValuesInner {
    pub fn new(
        custom_properties: Option<Arc<CustomPropertiesMap>>,
        writing_mode: WritingMode,
        flags: ComputedValueFlags,
        rules: Option<StrongRuleNode>,
        visited_style: Option<Arc<ComputedValues>>,
        % for style_struct in data.style_structs:
        ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
        % endfor
    ) -> Self {
        Self {
            custom_properties,
            writing_mode,
            rules,
            visited_style: visited_style.map(Arc::into_raw_offset),
            flags,
            % for style_struct in data.style_structs:
            ${style_struct.gecko_name}: Arc::into_raw_offset(${style_struct.ident}),
            % endfor
        }
    }

    fn to_outer(
        self,
        pseudo: Option<<&PseudoElement>,
    ) -> Arc<ComputedValues> {
        let pseudo_ty = match pseudo {
            Some(p) => p.pseudo_type(),
            None => structs::PseudoStyleType::NotPseudo,
        };
        let arc = unsafe {
            let arc: Arc<ComputedValues> = Arc::new(uninitialized());
            bindings::Gecko_ComputedStyle_Init(
                &arc.0 as *const _ as *mut _,
                &self,
                pseudo_ty,
            );
            // We're simulating a move by having C++ do a memcpy and then forgetting
            // it on this end.
            forget(self);
            arc
        };
        arc
    }
}

impl ops::Deref for ComputedValues {
    type Target = ComputedValuesInner;
    fn deref(&self) -> &ComputedValuesInner {
        &self.0.mSource
    }
}

impl ops::DerefMut for ComputedValues {
    fn deref_mut(&mut self) -> &mut ComputedValuesInner {
        &mut self.0.mSource
    }
}

impl ComputedValuesInner {
    /// Returns true if the value of the `content` property would make a
    /// pseudo-element not rendered.
    #[inline]
    pub fn ineffective_content_property(&self) -> bool {
        self.get_counters().ineffective_content_property()
    }

    % for style_struct in data.style_structs:
    #[inline]
    pub fn clone_${style_struct.name_lower}(&self) -> Arc<style_structs::${style_struct.name}> {
        Arc::from_raw_offset(self.${style_struct.gecko_name}.clone())
    }
    #[inline]
    pub fn get_${style_struct.name_lower}(&self) -> &style_structs::${style_struct.name} {
        &self.${style_struct.gecko_name}
    }


    pub fn ${style_struct.name_lower}_arc(&self) -> &RawOffsetArc<style_structs::${style_struct.name}> {
        &self.${style_struct.gecko_name}
    }

    #[inline]
    pub fn mutate_${style_struct.name_lower}(&mut self) -> &mut style_structs::${style_struct.name} {
        RawOffsetArc::make_mut(&mut self.${style_struct.gecko_name})
    }
    % endfor

    /// Gets the raw visited style. Useful for memory reporting.
    pub fn get_raw_visited_style(&self) -> &Option<RawOffsetArc<ComputedValues>> {
        &self.visited_style
    }

    #[allow(non_snake_case)]
    pub fn has_moz_binding(&self) -> bool {
        !self.get_box().gecko.mBinding.is_none()
    }
}

<%def name="declare_style_struct(style_struct)">
pub use crate::gecko_bindings::structs::mozilla::Gecko${style_struct.gecko_name} as ${style_struct.gecko_struct_name};
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
        ${set_gecko_property(gecko_ffi_name, "From::from(v)")}
    }
</%def>

<%def name="impl_simple_clone(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        From::from(self.gecko.${gecko_ffi_name}.clone())
    }
</%def>

<%def name="impl_simple_copy(ident, gecko_ffi_name, *kwargs)">
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name} = other.gecko.${gecko_ffi_name}.clone();
    }

    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }
</%def>

<%def name="impl_coord_copy(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name}.copy_from(&other.gecko.${gecko_ffi_name});
    }

    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }
</%def>

<%!
def get_gecko_property(ffi_name, self_param = "self"):
    return "%s.gecko.%s" % (self_param, ffi_name)

def set_gecko_property(ffi_name, expr):
    return "self.gecko.%s = %s;" % (ffi_name, expr)
%>

<%def name="impl_keyword_setter(ident, gecko_ffi_name, keyword, cast_type='u8')">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        use crate::properties::longhands::${ident}::computed_value::T as Keyword;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        let result = match v {
            % for value in keyword.values_for('gecko'):
                Keyword::${to_camel_case(value)} =>
                    structs::${keyword.gecko_constant(value)} ${keyword.maybe_cast(cast_type)},
            % endfor
        };
        ${set_gecko_property(gecko_ffi_name, "result")}
    }
</%def>

<%def name="impl_keyword_clone(ident, gecko_ffi_name, keyword, cast_type='u8')">
    // FIXME: We introduced non_upper_case_globals for -moz-appearance only
    //        since the prefix of Gecko value starts with ThemeWidgetType_NS_THEME.
    //        We should remove this after fix bug 1371809.
    #[allow(non_snake_case, non_upper_case_globals)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use crate::properties::longhands::${ident}::computed_value::T as Keyword;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts

        // Some constant macros in the gecko are defined as negative integer(e.g. font-stretch).
        // And they are convert to signed integer in Rust bindings. We need to cast then
        // as signed type when we have both signed/unsigned integer in order to use them
        // as match's arms.
        // Also, to use same implementation here we use casted constant if we have only singed values.
        % if keyword.gecko_enum_prefix is None:
        % for value in keyword.values_for('gecko'):
        const ${keyword.casted_constant_name(value, cast_type)} : ${cast_type} =
            structs::${keyword.gecko_constant(value)} as ${cast_type};
        % endfor

        match ${get_gecko_property(gecko_ffi_name)} as ${cast_type} {
            % for value in keyword.values_for('gecko'):
            ${keyword.casted_constant_name(value, cast_type)} => Keyword::${to_camel_case(value)},
            % endfor
            % if keyword.gecko_inexhaustive:
            _ => panic!("Found unexpected value in style struct for ${ident} property"),
            % endif
        }
        % else:
        match ${get_gecko_property(gecko_ffi_name)} {
            % for value in keyword.values_for('gecko'):
            structs::${keyword.gecko_constant(value)} => Keyword::${to_camel_case(value)},
            % endfor
            % if keyword.gecko_inexhaustive:
            _ => panic!("Found unexpected value in style struct for ${ident} property"),
            % endif
        }
        % endif
    }
</%def>

<%def name="impl_keyword(ident, gecko_ffi_name, keyword, cast_type='u8', **kwargs)">
<%call expr="impl_keyword_setter(ident, gecko_ffi_name, keyword, cast_type, **kwargs)"></%call>
<%call expr="impl_simple_copy(ident, gecko_ffi_name, **kwargs)"></%call>
<%call expr="impl_keyword_clone(ident, gecko_ffi_name, keyword, cast_type)"></%call>
</%def>

<%def name="impl_simple(ident, gecko_ffi_name)">
<%call expr="impl_simple_setter(ident, gecko_ffi_name)"></%call>
<%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
<%call expr="impl_simple_clone(ident, gecko_ffi_name)"></%call>
</%def>

<%def name="impl_absolute_length(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        ${set_gecko_property(gecko_ffi_name, "v.to_i32_au()")}
    }
    <%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        Au(self.gecko.${gecko_ffi_name}).into()
    }
</%def>

<%def name="impl_svg_length(ident, gecko_ffi_name)">
    // When context-value is used on an SVG length, the corresponding flag is
    // set on mContextFlags, and the length field is set to the initial value.

    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        use crate::values::generics::svg::SVGLength;
        use crate::gecko_bindings::structs::nsStyleSVG_${ident.upper()}_CONTEXT as CONTEXT_VALUE;
        let length = match v {
            SVGLength::LengthPercentage(length) => {
                self.gecko.mContextFlags &= !CONTEXT_VALUE;
                length
            }
            SVGLength::ContextValue => {
                self.gecko.mContextFlags |= CONTEXT_VALUE;
                match longhands::${ident}::get_initial_value() {
                    SVGLength::LengthPercentage(length) => length,
                    _ => unreachable!("Initial value should not be context-value"),
                }
            }
        };
        self.gecko.${gecko_ffi_name} = length;
    }

    pub fn copy_${ident}_from(&mut self, other: &Self) {
        use crate::gecko_bindings::structs::nsStyleSVG_${ident.upper()}_CONTEXT as CONTEXT_VALUE;
        self.gecko.${gecko_ffi_name} = other.gecko.${gecko_ffi_name};
        self.gecko.mContextFlags =
            (self.gecko.mContextFlags & !CONTEXT_VALUE) |
            (other.gecko.mContextFlags & CONTEXT_VALUE);
    }

    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use crate::values::generics::svg::SVGLength;
        use crate::gecko_bindings::structs::nsStyleSVG_${ident.upper()}_CONTEXT as CONTEXT_VALUE;
        if (self.gecko.mContextFlags & CONTEXT_VALUE) != 0 {
            return SVGLength::ContextValue;
        }
        SVGLength::LengthPercentage(self.gecko.${gecko_ffi_name})
    }
</%def>

<%def name="impl_svg_opacity(ident, gecko_ffi_name)">
    <% source_prefix = ident.split("_")[0].upper() + "_OPACITY_SOURCE" %>

    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        use crate::gecko_bindings::structs::nsStyleSVG_${source_prefix}_MASK as MASK;
        use crate::gecko_bindings::structs::nsStyleSVG_${source_prefix}_SHIFT as SHIFT;
        use crate::gecko_bindings::structs::nsStyleSVGOpacitySource::*;
        use crate::values::generics::svg::SVGOpacity;
        self.gecko.mContextFlags &= !MASK;
        match v {
            SVGOpacity::Opacity(opacity) => {
                self.gecko.mContextFlags |=
                    (eStyleSVGOpacitySource_Normal as u8) << SHIFT;
                self.gecko.${gecko_ffi_name} = opacity;
            }
            SVGOpacity::ContextFillOpacity => {
                self.gecko.mContextFlags |=
                    (eStyleSVGOpacitySource_ContextFillOpacity as u8) << SHIFT;
                self.gecko.${gecko_ffi_name} = 1.;
            }
            SVGOpacity::ContextStrokeOpacity => {
                self.gecko.mContextFlags |=
                    (eStyleSVGOpacitySource_ContextStrokeOpacity as u8) << SHIFT;
                self.gecko.${gecko_ffi_name} = 1.;
            }
        }
    }

    pub fn copy_${ident}_from(&mut self, other: &Self) {
        use crate::gecko_bindings::structs::nsStyleSVG_${source_prefix}_MASK as MASK;
        self.gecko.${gecko_ffi_name} = other.gecko.${gecko_ffi_name};
        self.gecko.mContextFlags =
            (self.gecko.mContextFlags & !MASK) |
            (other.gecko.mContextFlags & MASK);
    }

    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use crate::gecko_bindings::structs::nsStyleSVG_${source_prefix}_MASK as MASK;
        use crate::gecko_bindings::structs::nsStyleSVG_${source_prefix}_SHIFT as SHIFT;
        use crate::gecko_bindings::structs::nsStyleSVGOpacitySource::*;
        use crate::values::generics::svg::SVGOpacity;

        let source = (self.gecko.mContextFlags & MASK) >> SHIFT;
        if source == eStyleSVGOpacitySource_Normal as u8 {
            return SVGOpacity::Opacity(self.gecko.${gecko_ffi_name});
        } else {
            debug_assert_eq!(self.gecko.${gecko_ffi_name}, 1.0);
            if source == eStyleSVGOpacitySource_ContextFillOpacity as u8 {
                SVGOpacity::ContextFillOpacity
            } else {
                debug_assert_eq!(source, eStyleSVGOpacitySource_ContextStrokeOpacity as u8);
                SVGOpacity::ContextStrokeOpacity
            }
        }
    }
</%def>

<%def name="impl_svg_paint(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, mut v: longhands::${ident}::computed_value::T) {
        use crate::values::generics::svg::SVGPaintKind;
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
                    bindings::Gecko_nsStyleSVGPaint_SetURLValue(
                        paint,
                        &url
                    )
                }
            }
            SVGPaintKind::Color(color) => {
                paint.mType = nsStyleSVGPaintType::eStyleSVGPaintType_Color;
                unsafe {
                    *paint.mPaint.mColor.as_mut() = color.into();
                }
            }
        }

        paint.mFallbackType = match fallback {
            Some(Either::First(color)) => {
                paint.mFallbackColor = color.into();
                nsStyleSVGFallbackType::eStyleSVGFallbackType_Color
            },
            Some(Either::Second(_)) => {
                nsStyleSVGFallbackType::eStyleSVGFallbackType_None
            },
            None => nsStyleSVGFallbackType::eStyleSVGFallbackType_NotSet
        };
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
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use crate::values::generics::svg::{SVGPaint, SVGPaintKind};
        use self::structs::nsStyleSVGPaintType;
        use self::structs::nsStyleSVGFallbackType;
        let ref paint = ${get_gecko_property(gecko_ffi_name)};

        let fallback = match paint.mFallbackType {
            nsStyleSVGFallbackType::eStyleSVGFallbackType_Color => {
                Some(Either::First(paint.mFallbackColor.into()))
            },
            nsStyleSVGFallbackType::eStyleSVGFallbackType_None => {
                Some(Either::Second(None_))
            },
            nsStyleSVGFallbackType::eStyleSVGFallbackType_NotSet => None,
        };

        let kind = match paint.mType {
            nsStyleSVGPaintType::eStyleSVGPaintType_None => SVGPaintKind::None,
            nsStyleSVGPaintType::eStyleSVGPaintType_ContextFill => SVGPaintKind::ContextFill,
            nsStyleSVGPaintType::eStyleSVGPaintType_ContextStroke => SVGPaintKind::ContextStroke,
            nsStyleSVGPaintType::eStyleSVGPaintType_Server => {
                SVGPaintKind::PaintServer(unsafe {
                    paint.mPaint.mPaintServer.as_ref().clone()
                })
            }
            nsStyleSVGPaintType::eStyleSVGPaintType_Color => {
                let col = unsafe { *paint.mPaint.mColor.as_ref() };
                SVGPaintKind::Color(col.into())
            }
        };
        SVGPaint {
            kind: kind,
            fallback: fallback,
        }
    }
</%def>

<%def name="impl_non_negative_length(ident, gecko_ffi_name, inherit_from=None,
                                     round_to_pixels=False)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        let value = {
            % if round_to_pixels:
            let au_per_device_px = Au(self.gecko.mTwipsPerPixel);
            round_border_to_device_pixels(Au::from(v), au_per_device_px).0
            % else:
            v.0.to_i32_au()
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
        //
        // FIXME(emilio): We could clean this up a bit special-casing the reset_
        // function below.
        self.gecko.${gecko_ffi_name} = other.gecko.${inherit_from};
        % else:
        self.gecko.${gecko_ffi_name} = other.gecko.${gecko_ffi_name};
        % endif
    }

    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        Au(self.gecko.${gecko_ffi_name}).into()
    }
</%def>

<%def name="impl_split_style_coord(ident, gecko_ffi_name, index)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        self.gecko.${gecko_ffi_name}.${index} = v;
    }
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name}.${index} =
            other.gecko.${gecko_ffi_name}.${index};
    }
    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        self.gecko.${gecko_ffi_name}.${index}
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

    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }
</%def>

<%def name="impl_corner_style_coord(ident, gecko_ffi_name, corner)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        self.gecko.${gecko_ffi_name}.${corner} = v;
    }
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name}.${corner} =
            other.gecko.${gecko_ffi_name}.${corner};
    }
    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        self.gecko.${gecko_ffi_name}.${corner}
    }
</%def>

<%def name="impl_logical(name, **kwargs)">
    ${helpers.logical_setter(name)}
</%def>

<%def name="impl_style_struct(style_struct)">
impl ${style_struct.gecko_struct_name} {
    #[allow(dead_code, unused_variables)]
    pub fn default(document: &structs::Document) -> Arc<Self> {
        let mut result = Arc::new(${style_struct.gecko_struct_name} { gecko: ManuallyDrop::new(unsafe { zeroed() }) });
        unsafe {
            Gecko_Construct_Default_${style_struct.gecko_ffi_name}(
                &mut *Arc::get_mut(&mut result).unwrap().gecko,
                document,
            );
        }
        result
    }
}
impl Drop for ${style_struct.gecko_struct_name} {
    fn drop(&mut self) {
        unsafe {
            Gecko_Destroy_${style_struct.gecko_ffi_name}(&mut *self.gecko);
        }
    }
}
impl Clone for ${style_struct.gecko_struct_name} {
    fn clone(&self) -> Self {
        unsafe {
            let mut result = ${style_struct.gecko_struct_name} { gecko: ManuallyDrop::new(zeroed()) };
            Gecko_CopyConstruct_${style_struct.gecko_ffi_name}(&mut *result.gecko, &*self.gecko);
            result
        }
    }
}

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

<%def name="impl_font_settings(ident, gecko_type, tag_type, value_type, gecko_value_type)">
    <%
    gecko_ffi_name = to_camel_case_lower(ident)
    %>

    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        let iter = v.0.iter().map(|other| structs::${gecko_type} {
            mTag: other.tag.0,
            mValue: other.value as ${gecko_value_type},
        });
        self.gecko.mFont.${gecko_ffi_name}.assign_from_iter_pod(iter);
    }

    pub fn copy_${ident}_from(&mut self, other: &Self) {
        let iter = other.gecko.mFont.${gecko_ffi_name}.iter().map(|s| *s);
        self.gecko.mFont.${gecko_ffi_name}.assign_from_iter_pod(iter);
    }

    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use crate::values::generics::font::{FontSettings, FontTag, ${tag_type}};

        FontSettings(
            self.gecko.mFont.${gecko_ffi_name}.iter().map(|gecko_font_setting| {
                ${tag_type} {
                    tag: FontTag(gecko_font_setting.mTag),
                    value: gecko_font_setting.mValue as ${value_type},
                }
            }).collect::<Vec<_>>().into_boxed_slice()
        )
    }
</%def>

<%def name="impl_trait(style_struct_name, skip_longhands='')">
<%
    style_struct = next(x for x in data.style_structs if x.name == style_struct_name)
    longhands = [x for x in style_struct.longhands
                if not (skip_longhands == "*" or x.name in skip_longhands.split())]

    # Types used with predefined_type()-defined properties that we can auto-generate.
    predefined_types = {
        "MozScriptMinSize": impl_absolute_length,
        "SVGLength": impl_svg_length,
        "SVGOpacity": impl_svg_opacity,
        "SVGPaint": impl_svg_paint,
        "SVGWidth": impl_svg_length,
    }

    def longhand_method(longhand):
        args = dict(ident=longhand.ident, gecko_ffi_name=longhand.gecko_ffi_name)

        # get the method and pass additional keyword or type-specific arguments
        if longhand.logical:
            method = impl_logical
            args.update(name=longhand.name)
        elif longhand.keyword:
            method = impl_keyword
            args.update(keyword=longhand.keyword)
            if "font" in longhand.ident:
                args.update(cast_type=longhand.cast_type)
        elif longhand.predefined_type in predefined_types:
            method = predefined_types[longhand.predefined_type]
        else:
            method = impl_simple

        method(**args)
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
    for longhand in longhands:
        longhand_method(longhand)
    %>
}
</%def>

<%!
class Side(object):
    def __init__(self, name, index):
        self.name = name
        self.ident = name.lower()
        self.index = index

class GridLine(object):
    def __init__(self, name):
        self.ident = "grid-" + name.lower()
        self.name = self.ident.replace('-', '_')
        self.gecko = "m" + to_camel_case(self.ident)

SIDES = [Side("Top", 0), Side("Right", 1), Side("Bottom", 2), Side("Left", 3)]
CORNERS = ["top_left", "top_right", "bottom_right", "bottom_left"]
GRID_LINES = map(GridLine, ["row-start", "row-end", "column-start", "column-end"])
%>

#[allow(dead_code)]
fn static_assert() {
    // Note: using the above technique with an enum hits a rust bug when |structs| is in a different crate.
    % for side in SIDES:
    { const DETAIL: u32 = [0][(structs::Side::eSide${side.name} as usize != ${side.index}) as usize]; let _ = DETAIL; }
    % endfor
}


<% skip_border_longhands = " ".join(["border-{0}-{1}".format(x.ident, y)
                                     for x in SIDES
                                     for y in ["color", "style", "width"]] +
                                    ["border-{0}-radius".format(x.replace("_", "-"))
                                     for x in CORNERS]) %>

<%self:impl_trait style_struct_name="Border"
                  skip_longhands="${skip_border_longhands} border-image-source
                                  border-image-repeat">
    % for side in SIDES:
    pub fn set_border_${side.ident}_style(&mut self, v: BorderStyle) {
        self.gecko.mBorderStyle[${side.index}] = v;

        // This is needed because the initial mComputedBorder value is set to
        // zero.
        //
        // In order to compute stuff, we start from the initial struct, and keep
        // going down the tree applying properties.
        //
        // That means, effectively, that when we set border-style to something
        // non-hidden, we should use the initial border instead.
        //
        // Servo stores the initial border-width in the initial struct, and then
        // adjusts as needed in the fixup phase. This means that the initial
        // struct is technically not valid without fixups, and that you lose
        // pretty much any sharing of the initial struct, which is kind of
        // unfortunate.
        //
        // Gecko has two fields for this, one that stores the "specified"
        // border, and other that stores the actual computed one. That means
        // that when we set border-style, border-width may change and we need to
        // sync back to the specified one. This is what this function does.
        //
        // Note that this doesn't impose any dependency in the order of
        // computation of the properties. This is only relevant if border-style
        // is specified, but border-width isn't. If border-width is specified at
        // some point, the two mBorder and mComputedBorder fields would be the
        // same already.
        //
        // Once we're here, we know that we'll run style fixups, so it's fine to
        // just copy the specified border here, we'll adjust it if it's
        // incorrect later.
        self.gecko.mComputedBorder.${side.ident} = self.gecko.mBorder.${side.ident};
    }

    pub fn copy_border_${side.ident}_style_from(&mut self, other: &Self) {
        self.gecko.mBorderStyle[${side.index}] = other.gecko.mBorderStyle[${side.index}];
        self.gecko.mComputedBorder.${side.ident} = self.gecko.mBorder.${side.ident};
    }

    pub fn reset_border_${side.ident}_style(&mut self, other: &Self) {
        self.copy_border_${side.ident}_style_from(other);
    }

    #[inline]
    pub fn clone_border_${side.ident}_style(&self) -> BorderStyle {
        self.gecko.mBorderStyle[${side.index}]
    }

    <% impl_simple("border_%s_color" % side.ident, "mBorder%sColor" % side.name) %>

    <% impl_non_negative_length("border_%s_width" % side.ident,
                                "mComputedBorder.%s" % side.ident,
                                inherit_from="mBorder.%s" % side.ident,
                                round_to_pixels=True) %>

    pub fn border_${side.ident}_has_nonzero_width(&self) -> bool {
        self.gecko.mComputedBorder.${side.ident} != 0
    }
    % endfor

    % for corner in CORNERS:
    <% impl_corner_style_coord("border_%s_radius" % corner,
                               "mBorderRadius",
                               corner) %>
    % endfor

    pub fn set_border_image_source(&mut self, image: longhands::border_image_source::computed_value::T) {
        unsafe {
            // Prevent leaking of the last elements we did set
            Gecko_SetNullImageValue(&mut self.gecko.mBorderImageSource);
        }

        if let ImageLayer::Image(image) = image {
            self.gecko.mBorderImageSource.set(image);
        }
    }

    pub fn copy_border_image_source_from(&mut self, other: &Self) {
        unsafe {
            Gecko_CopyImageValueFrom(&mut self.gecko.mBorderImageSource,
                                     &other.gecko.mBorderImageSource);
        }
    }

    pub fn reset_border_image_source(&mut self, other: &Self) {
        self.copy_border_image_source_from(other)
    }

    pub fn clone_border_image_source(&self) -> longhands::border_image_source::computed_value::T {
        match unsafe { self.gecko.mBorderImageSource.into_image() } {
            Some(image) => ImageLayer::Image(image),
            None => ImageLayer::None,
        }
    }

    <%
    border_image_repeat_keywords = ["Stretch", "Repeat", "Round", "Space"]
    %>

    pub fn set_border_image_repeat(&mut self, v: longhands::border_image_repeat::computed_value::T) {
        use crate::values::specified::border::BorderImageRepeatKeyword;
        use crate::gecko_bindings::structs::StyleBorderImageRepeat;

        % for i, side in enumerate(["H", "V"]):
            self.gecko.mBorderImageRepeat${side} = match v.${i} {
                % for keyword in border_image_repeat_keywords:
                BorderImageRepeatKeyword::${keyword} => StyleBorderImageRepeat::${keyword},
                % endfor
            };
        % endfor
    }

    pub fn copy_border_image_repeat_from(&mut self, other: &Self) {
        self.gecko.mBorderImageRepeatH = other.gecko.mBorderImageRepeatH;
        self.gecko.mBorderImageRepeatV = other.gecko.mBorderImageRepeatV;
    }

    pub fn reset_border_image_repeat(&mut self, other: &Self) {
        self.copy_border_image_repeat_from(other)
    }

    pub fn clone_border_image_repeat(&self) -> longhands::border_image_repeat::computed_value::T {
        use crate::values::specified::border::BorderImageRepeatKeyword;
        use crate::gecko_bindings::structs::StyleBorderImageRepeat;

        % for side in ["H", "V"]:
        let servo_${side.lower()} = match self.gecko.mBorderImageRepeat${side} {
            % for keyword in border_image_repeat_keywords:
            StyleBorderImageRepeat::${keyword} => BorderImageRepeatKeyword::${keyword},
            % endfor
        };
        % endfor
        longhands::border_image_repeat::computed_value::T(servo_h, servo_v)
    }
</%self:impl_trait>

<% skip_scroll_margin_longhands = " ".join(["scroll-margin-%s" % x.ident for x in SIDES]) %>
<% skip_margin_longhands = " ".join(["margin-%s" % x.ident for x in SIDES]) %>
<%self:impl_trait style_struct_name="Margin"
                  skip_longhands="${skip_margin_longhands}
                                  ${skip_scroll_margin_longhands}">

    % for side in SIDES:
    <% impl_split_style_coord("margin_%s" % side.ident,
                              "mMargin",
                              side.index) %>
    <% impl_split_style_coord("scroll_margin_%s" % side.ident,
                              "mScrollMargin",
                              side.index) %>
    % endfor
</%self:impl_trait>

<% skip_scroll_padding_longhands = " ".join(["scroll-padding-%s" % x.ident for x in SIDES]) %>
<% skip_padding_longhands = " ".join(["padding-%s" % x.ident for x in SIDES]) %>
<%self:impl_trait style_struct_name="Padding"
                  skip_longhands="${skip_padding_longhands}
                                  ${skip_scroll_padding_longhands}">

    % for side in SIDES:
    <% impl_split_style_coord("padding_%s" % side.ident,
                              "mPadding",
                              side.index) %>
    <% impl_split_style_coord("scroll_padding_%s" % side.ident, "mScrollPadding", side.index) %>
    % endfor
</%self:impl_trait>

<% skip_position_longhands = " ".join(x.ident for x in SIDES + GRID_LINES) %>
<%self:impl_trait style_struct_name="Position"
                  skip_longhands="${skip_position_longhands} order
                                  align-content justify-content align-self
                                  justify-self align-items justify-items
                                  grid-auto-rows grid-auto-columns
                                  grid-auto-flow grid-template-rows
                                  grid-template-columns">
    % for side in SIDES:
    <% impl_split_style_coord(side.ident, "mOffset", side.index) %>
    % endfor

    % for kind in ["align", "justify"]:
    ${impl_simple_type_with_conversion(kind + "_content")}
    ${impl_simple_type_with_conversion(kind + "_self")}
    % endfor
    ${impl_simple_type_with_conversion("align_items")}

    pub fn set_justify_items(&mut self, v: longhands::justify_items::computed_value::T) {
        self.gecko.mSpecifiedJustifyItems = v.specified.into();
        self.set_computed_justify_items(v.computed);
    }

    pub fn set_computed_justify_items(&mut self, v: values::specified::JustifyItems) {
        debug_assert_ne!(v.0, crate::values::specified::align::AlignFlags::LEGACY);
        self.gecko.mJustifyItems = v.into();
    }

    pub fn reset_justify_items(&mut self, reset_style: &Self) {
        self.gecko.mJustifyItems = reset_style.gecko.mJustifyItems;
        self.gecko.mSpecifiedJustifyItems = reset_style.gecko.mSpecifiedJustifyItems;
    }

    pub fn copy_justify_items_from(&mut self, other: &Self) {
        self.gecko.mJustifyItems = other.gecko.mJustifyItems;
        self.gecko.mSpecifiedJustifyItems = other.gecko.mJustifyItems;
    }

    pub fn clone_justify_items(&self) -> longhands::justify_items::computed_value::T {
        longhands::justify_items::computed_value::T {
            computed: self.gecko.mJustifyItems.into(),
            specified: self.gecko.mSpecifiedJustifyItems.into(),
        }
    }

    pub fn set_order(&mut self, v: longhands::order::computed_value::T) {
        self.gecko.mOrder = v;
    }

    pub fn clone_order(&self) -> longhands::order::computed_value::T {
        self.gecko.mOrder
    }

    ${impl_simple_copy('order', 'mOrder')}

    % for value in GRID_LINES:
    pub fn set_${value.name}(&mut self, v: longhands::${value.name}::computed_value::T) {
        use crate::gecko_bindings::structs::{nsStyleGridLine_kMinLine, nsStyleGridLine_kMaxLine};

        let line = &mut self.gecko.${value.gecko};
        line.mLineName.set_move(unsafe {
            RefPtr::from_addrefed(match v.ident {
                Some(i) => i.0,
                None => atom!(""),
            }.into_addrefed())
        });
        line.mHasSpan = v.is_span;
        if let Some(integer) = v.line_num {
            // clamping the integer between a range
            line.mInteger = cmp::max(
                nsStyleGridLine_kMinLine,
                cmp::min(integer, nsStyleGridLine_kMaxLine),
            );
        }
    }

    pub fn copy_${value.name}_from(&mut self, other: &Self) {
        self.gecko.${value.gecko}.mHasSpan = other.gecko.${value.gecko}.mHasSpan;
        self.gecko.${value.gecko}.mInteger = other.gecko.${value.gecko}.mInteger;
        unsafe {
            self.gecko.${value.gecko}.mLineName.set(&other.gecko.${value.gecko}.mLineName);
        }
    }

    pub fn reset_${value.name}(&mut self, other: &Self) {
        self.copy_${value.name}_from(other)
    }

    pub fn clone_${value.name}(&self) -> longhands::${value.name}::computed_value::T {
        use crate::gecko_bindings::structs::{nsStyleGridLine_kMinLine, nsStyleGridLine_kMaxLine};

        longhands::${value.name}::computed_value::T {
            is_span: self.gecko.${value.gecko}.mHasSpan,
            ident: {
                let name = unsafe { Atom::from_raw(self.gecko.${value.gecko}.mLineName.mRawPtr) };
                if name == atom!("") {
                    None
                } else {
                    Some(CustomIdent(name))
                }
            },
            line_num:
                if self.gecko.${value.gecko}.mInteger == 0 {
                    None
                } else {
                    debug_assert!(nsStyleGridLine_kMinLine <= self.gecko.${value.gecko}.mInteger);
                    debug_assert!(self.gecko.${value.gecko}.mInteger <= nsStyleGridLine_kMaxLine);
                    Some(self.gecko.${value.gecko}.mInteger)
                },
        }
    }
    % endfor

    % for kind in ["rows", "columns"]:
    pub fn set_grid_auto_${kind}(&mut self, v: longhands::grid_auto_${kind}::computed_value::T) {
        let gecko = &mut *self.gecko;
        v.to_gecko_style_coords(&mut gecko.mGridAuto${kind.title()}Min,
                                &mut gecko.mGridAuto${kind.title()}Max)
    }

    pub fn copy_grid_auto_${kind}_from(&mut self, other: &Self) {
        self.gecko.mGridAuto${kind.title()}Min.copy_from(&other.gecko.mGridAuto${kind.title()}Min);
        self.gecko.mGridAuto${kind.title()}Max.copy_from(&other.gecko.mGridAuto${kind.title()}Max);
    }

    pub fn reset_grid_auto_${kind}(&mut self, other: &Self) {
        self.copy_grid_auto_${kind}_from(other)
    }

    pub fn clone_grid_auto_${kind}(&self) -> longhands::grid_auto_${kind}::computed_value::T {
        crate::values::generics::grid::TrackSize::from_gecko_style_coords(&self.gecko.mGridAuto${kind.title()}Min,
                                                                     &self.gecko.mGridAuto${kind.title()}Max)
    }

    pub fn set_grid_template_${kind}(&mut self, v: longhands::grid_template_${kind}::computed_value::T) {
        <% self_grid = "self.gecko.mGridTemplate%s" % kind.title() %>
        use crate::gecko_bindings::structs::{nsTArray, nsStyleGridLine_kMaxLine};
        use std::usize;
        use crate::values::CustomIdent;
        use crate::values::generics::grid::TrackListType::Auto;
        use crate::values::generics::grid::{GridTemplateComponent, RepeatCount};

        #[inline]
        fn set_line_names(servo_names: &[CustomIdent], gecko_names: &mut nsTArray<structs::RefPtr<structs::nsAtom>>) {
            unsafe {
                bindings::Gecko_ResizeAtomArray(gecko_names, servo_names.len() as u32);
            }

            for (servo_name, gecko_name) in servo_names.iter().zip(gecko_names.iter_mut()) {
                gecko_name.set_move(unsafe {
                    RefPtr::from_addrefed(servo_name.0.clone().into_addrefed())
                });
            }
        }

        let max_lines = nsStyleGridLine_kMaxLine as usize - 1;      // for accounting the final <line-names>

        let result = match v {
            GridTemplateComponent::None => ptr::null_mut(),
            GridTemplateComponent::TrackList(track) => {
                let mut num_values = track.values.len();
                if let Auto(_) = track.list_type {
                    num_values += 1;
                }

                num_values = cmp::min(num_values, max_lines);
                let value = unsafe {
                    bindings::Gecko_CreateStyleGridTemplate(num_values as u32,
                                                            (num_values + 1) as u32).as_mut().unwrap()
                };

                let mut auto_idx = usize::MAX;
                let mut auto_track_size = None;
                if let Auto(idx) = track.list_type {
                    auto_idx = idx as usize;
                    let auto_repeat = track.auto_repeat.as_ref().expect("expected <auto-track-repeat> value");

                    if auto_repeat.count == RepeatCount::AutoFill {
                        value.set_mIsAutoFill(true);
                    }

                    value.mRepeatAutoIndex = idx as i16;
                    // NOTE: Gecko supports only one set of values in <auto-repeat>
                    // i.e., it can only take repeat(auto-fill, [a] 10px [b]), and no more.
                    set_line_names(&auto_repeat.line_names[0], &mut value.mRepeatAutoLineNameListBefore);
                    set_line_names(&auto_repeat.line_names[1], &mut value.mRepeatAutoLineNameListAfter);
                    auto_track_size = Some(auto_repeat.track_sizes.get(0).unwrap().clone());
                } else {
                    unsafe {
                        bindings::Gecko_ResizeAtomArray(
                            &mut value.mRepeatAutoLineNameListBefore, 0);
                        bindings::Gecko_ResizeAtomArray(
                            &mut value.mRepeatAutoLineNameListAfter, 0);
                    }
                }

                let mut line_names = track.line_names.into_iter();
                let mut values_iter = track.values.into_iter();
                {
                    let min_max_iter = value.mMinTrackSizingFunctions.iter_mut()
                                            .zip(value.mMaxTrackSizingFunctions.iter_mut());
                    for (i, (gecko_min, gecko_max)) in min_max_iter.enumerate().take(max_lines) {
                        let name_list = line_names.next().expect("expected line-names");
                        set_line_names(&name_list, &mut value.mLineNameLists[i]);
                        if i == auto_idx {
                            let track_size = auto_track_size.take()
                                .expect("expected <track-size> for <auto-track-repeat>");
                            track_size.to_gecko_style_coords(gecko_min, gecko_max);
                            continue
                        }

                        let track_size = values_iter.next().expect("expected <track-size> value");
                        track_size.to_gecko_style_coords(gecko_min, gecko_max);
                    }
                }

                let final_names = line_names.next().unwrap();
                set_line_names(&final_names, value.mLineNameLists.last_mut().unwrap());

                value
            },
            GridTemplateComponent::Subgrid(list) => {
                let names_length = match list.fill_idx {
                    Some(_) => list.names.len() - 1,
                    None => list.names.len(),
                };
                let num_values = cmp::min(names_length, max_lines + 1);
                let value = unsafe {
                    bindings::Gecko_CreateStyleGridTemplate(0, num_values as u32).as_mut().unwrap()
                };
                value.set_mIsSubgrid(true);

                let mut names = list.names.into_vec();
                if let Some(idx) = list.fill_idx {
                    value.set_mIsAutoFill(true);
                    value.mRepeatAutoIndex = idx as i16;
                    set_line_names(&names.swap_remove(idx as usize),
                                   &mut value.mRepeatAutoLineNameListBefore);
                }

                for (servo_names, gecko_names) in names.iter().zip(value.mLineNameLists.iter_mut()) {
                    set_line_names(servo_names, gecko_names);
                }

                value
            },
        };

        unsafe { bindings::Gecko_SetStyleGridTemplate(&mut ${self_grid}, result); }
    }

    pub fn copy_grid_template_${kind}_from(&mut self, other: &Self) {
        unsafe {
            bindings::Gecko_CopyStyleGridTemplateValues(&mut ${self_grid},
                                                        other.gecko.mGridTemplate${kind.title()}.mPtr);
        }
    }

    pub fn reset_grid_template_${kind}(&mut self, other: &Self) {
        self.copy_grid_template_${kind}_from(other)
    }

    pub fn clone_grid_template_${kind}(&self) -> longhands::grid_template_${kind}::computed_value::T {
        <% self_grid = "self.gecko.mGridTemplate%s" % kind.title() %>
        use crate::gecko_bindings::structs::nsTArray;
        use crate::values::CustomIdent;
        use crate::values::generics::grid::{GridTemplateComponent, LineNameList, RepeatCount};
        use crate::values::generics::grid::{TrackList, TrackListType, TrackListValue, TrackRepeat, TrackSize};

        let value = match unsafe { ${self_grid}.mPtr.as_ref() } {
            None => return GridTemplateComponent::None,
            Some(value) => value,
        };

        #[inline]
        fn to_boxed_customident_slice(gecko_names: &nsTArray<structs::RefPtr<structs::nsAtom>>) -> Box<[CustomIdent]> {
            let idents: Vec<CustomIdent> = gecko_names.iter().map(|gecko_name| {
                CustomIdent(unsafe { Atom::from_raw(gecko_name.mRawPtr) })
            }).collect();
            idents.into_boxed_slice()
        }

        #[inline]
        fn to_line_names_vec(
            gecko_line_names: &nsTArray<nsTArray<structs::RefPtr<structs::nsAtom>>>,
        ) -> Vec<Box<[CustomIdent]>> {
            gecko_line_names.iter().map(|gecko_names| {
                to_boxed_customident_slice(gecko_names)
            }).collect()
        }

        let repeat_auto_index = value.mRepeatAutoIndex as usize;
        if value.mIsSubgrid() {
            let mut names_vec = to_line_names_vec(&value.mLineNameLists);
            let fill_idx = if value.mIsAutoFill() {
                names_vec.insert(
                    repeat_auto_index,
                    to_boxed_customident_slice(&value.mRepeatAutoLineNameListBefore));
                Some(repeat_auto_index as u32)
            } else {
                None
            };
            let names = names_vec.into_boxed_slice();

            GridTemplateComponent::Subgrid(LineNameList{names, fill_idx})
        } else {
            let mut auto_repeat = None;
            let mut list_type = TrackListType::Normal;
            let line_names = to_line_names_vec(&value.mLineNameLists).into_boxed_slice();
            let mut values = Vec::with_capacity(value.mMinTrackSizingFunctions.len());

            let min_max_iter = value.mMinTrackSizingFunctions.iter()
                .zip(value.mMaxTrackSizingFunctions.iter());
            for (i, (gecko_min, gecko_max)) in min_max_iter.enumerate() {
                let track_size = TrackSize::from_gecko_style_coords(gecko_min, gecko_max);

                if i == repeat_auto_index {
                    list_type = TrackListType::Auto(repeat_auto_index as u16);

                    let count = if value.mIsAutoFill() {
                        RepeatCount::AutoFill
                    } else {
                        RepeatCount::AutoFit
                    };

                    let line_names = {
                        let mut vec: Vec<Box<[CustomIdent]>> = Vec::with_capacity(2);
                        vec.push(to_boxed_customident_slice(
                            &value.mRepeatAutoLineNameListBefore));
                        vec.push(to_boxed_customident_slice(
                            &value.mRepeatAutoLineNameListAfter));
                        vec.into_boxed_slice()
                    };

                    let track_sizes = vec!(track_size);

                    auto_repeat = Some(TrackRepeat{count, line_names, track_sizes});
                } else {
                    values.push(TrackListValue::TrackSize(track_size));
                }
            }

            GridTemplateComponent::TrackList(TrackList{list_type, values, line_names, auto_repeat})
        }
    }
    % endfor

    ${impl_simple_type_with_conversion("grid_auto_flow")}
</%self:impl_trait>

<% skip_outline_longhands = " ".join("outline-style outline-width".split() +
                                     ["-moz-outline-radius-{0}".format(x.replace("_", ""))
                                      for x in CORNERS]) %>
<%self:impl_trait style_struct_name="Outline"
                  skip_longhands="${skip_outline_longhands}">

    pub fn set_outline_style(&mut self, v: longhands::outline_style::computed_value::T) {
        self.gecko.mOutlineStyle = v;
        // NB: This is needed to correctly handling the initial value of
        // outline-width when outline-style changes, see the
        // update_border_${side.ident} comment for more details.
        self.gecko.mActualOutlineWidth = self.gecko.mOutlineWidth;
    }

    pub fn copy_outline_style_from(&mut self, other: &Self) {
        // FIXME(emilio): Why doesn't this need to reset mActualOutlineWidth?
        // Looks fishy.
        self.gecko.mOutlineStyle = other.gecko.mOutlineStyle;
    }

    pub fn reset_outline_style(&mut self, other: &Self) {
        self.copy_outline_style_from(other)
    }

    pub fn clone_outline_style(&self) -> longhands::outline_style::computed_value::T {
        self.gecko.mOutlineStyle.clone()
    }

    <% impl_non_negative_length("outline_width", "mActualOutlineWidth",
                                inherit_from="mOutlineWidth",
                                round_to_pixels=True) %>

    % for corner in CORNERS:
    <% impl_corner_style_coord("_moz_outline_radius_%s" % corner.replace("_", ""),
                               "mOutlineRadius",
                               corner) %>
    % endfor

    pub fn outline_has_nonzero_width(&self) -> bool {
        self.gecko.mActualOutlineWidth != 0
    }
</%self:impl_trait>

<%
    skip_font_longhands = """font-family font-size font-size-adjust font-weight
                             font-style font-stretch -moz-script-level
                             font-synthesis -x-lang font-variant-alternates
                             font-variant-east-asian font-variant-ligatures
                             font-variant-numeric font-language-override
                             font-feature-settings font-variation-settings
                             -moz-min-font-size-ratio -x-text-zoom"""
%>
<%self:impl_trait style_struct_name="Font"
    skip_longhands="${skip_font_longhands}">

    // Negative numbers are invalid at parse time, but <integer> is still an
    // i32.
    <% impl_font_settings("font_feature_settings", "gfxFontFeature", "FeatureTagValue", "i32", "u32") %>
    <% impl_font_settings("font_variation_settings", "gfxFontVariation", "VariationValue", "f32", "f32") %>

    pub fn set_font_family(&mut self, v: longhands::font_family::computed_value::T) {
        use crate::values::computed::font::GenericFontFamily;

        let is_system_font = v.is_system_font;
        self.gecko.mFont.systemFont = is_system_font;
        self.gecko.mGenericID = if is_system_font {
            GenericFontFamily::None
        } else {
            v.families.single_generic().unwrap_or(GenericFontFamily::None)
        };
        self.gecko.mFont.fontlist.mFontlist.mBasePtr.set_move(
            v.families.shared_font_list().clone()
        );
        // Fixed-up if needed in Cascade::fixup_font_stuff.
        self.gecko.mFont.fontlist.mDefaultFontType = GenericFontFamily::None;
    }

    pub fn copy_font_family_from(&mut self, other: &Self) {
        unsafe { Gecko_CopyFontFamilyFrom(&mut self.gecko.mFont, &other.gecko.mFont); }
        self.gecko.mGenericID = other.gecko.mGenericID;
        self.gecko.mFont.systemFont = other.gecko.mFont.systemFont;
    }

    pub fn reset_font_family(&mut self, other: &Self) {
        self.copy_font_family_from(other)
    }

    pub fn clone_font_family(&self) -> longhands::font_family::computed_value::T {
        use crate::values::computed::font::{FontFamily, SingleFontFamily, FontFamilyList};

        let fontlist = &self.gecko.mFont.fontlist;
        let shared_fontlist = unsafe { fontlist.mFontlist.mBasePtr.to_safe() };

        let families = if shared_fontlist.mNames.is_empty() {
            let default = SingleFontFamily::Generic(fontlist.mDefaultFontType);
            FontFamilyList::new(Box::new([default]))
        } else {
            FontFamilyList::SharedFontList(shared_fontlist)
        };

        FontFamily {
            families,
            is_system_font: self.gecko.mFont.systemFont,
        }
    }

    pub fn unzoom_fonts(&mut self, device: &Device) {
        self.gecko.mSize = device.unzoom_text(Au(self.gecko.mSize)).0;
        self.gecko.mScriptUnconstrainedSize = device.unzoom_text(Au(self.gecko.mScriptUnconstrainedSize)).0;
        self.gecko.mFont.size = device.unzoom_text(Au(self.gecko.mFont.size)).0;
    }

    pub fn copy_font_size_from(&mut self, other: &Self) {
        self.gecko.mScriptUnconstrainedSize = other.gecko.mScriptUnconstrainedSize;

        self.gecko.mSize = other.gecko.mScriptUnconstrainedSize;
        self.gecko.mFont.size = other.gecko.mSize;
        self.gecko.mFontSizeKeyword = other.gecko.mFontSizeKeyword;

        // TODO(emilio): Should we really copy over these two?
        self.gecko.mFontSizeFactor = other.gecko.mFontSizeFactor;
        self.gecko.mFontSizeOffset = other.gecko.mFontSizeOffset;
    }

    pub fn reset_font_size(&mut self, other: &Self) {
        self.copy_font_size_from(other)
    }

    pub fn set_font_size(&mut self, v: FontSize) {
        use crate::values::generics::font::KeywordSize;

        let size = v.size();
        self.gecko.mScriptUnconstrainedSize = size.0;

        // These two may be changed from Cascade::fixup_font_stuff.
        self.gecko.mSize = size.0;
        self.gecko.mFont.size = size.0;

        if let Some(info) = v.keyword_info {
            self.gecko.mFontSizeKeyword = match info.kw {
                KeywordSize::XXSmall => structs::NS_STYLE_FONT_SIZE_XXSMALL,
                KeywordSize::XSmall => structs::NS_STYLE_FONT_SIZE_XSMALL,
                KeywordSize::Small => structs::NS_STYLE_FONT_SIZE_SMALL,
                KeywordSize::Medium => structs::NS_STYLE_FONT_SIZE_MEDIUM,
                KeywordSize::Large => structs::NS_STYLE_FONT_SIZE_LARGE,
                KeywordSize::XLarge => structs::NS_STYLE_FONT_SIZE_XLARGE,
                KeywordSize::XXLarge => structs::NS_STYLE_FONT_SIZE_XXLARGE,
                KeywordSize::XXXLarge => structs::NS_STYLE_FONT_SIZE_XXXLARGE,
            } as u8;
            self.gecko.mFontSizeFactor = info.factor;
            self.gecko.mFontSizeOffset = info.offset.0.to_i32_au();
        } else {
            self.gecko.mFontSizeKeyword = structs::NS_STYLE_FONT_SIZE_NO_KEYWORD as u8;
            self.gecko.mFontSizeFactor = 1.;
            self.gecko.mFontSizeOffset = 0;
        }
    }

    pub fn clone_font_size(&self) -> FontSize {
        use crate::values::generics::font::{KeywordInfo, KeywordSize};
        let size = Au(self.gecko.mSize).into();
        let kw = match self.gecko.mFontSizeKeyword as u32 {
            structs::NS_STYLE_FONT_SIZE_XXSMALL => KeywordSize::XXSmall,
            structs::NS_STYLE_FONT_SIZE_XSMALL => KeywordSize::XSmall,
            structs::NS_STYLE_FONT_SIZE_SMALL => KeywordSize::Small,
            structs::NS_STYLE_FONT_SIZE_MEDIUM => KeywordSize::Medium,
            structs::NS_STYLE_FONT_SIZE_LARGE => KeywordSize::Large,
            structs::NS_STYLE_FONT_SIZE_XLARGE => KeywordSize::XLarge,
            structs::NS_STYLE_FONT_SIZE_XXLARGE => KeywordSize::XXLarge,
            structs::NS_STYLE_FONT_SIZE_XXXLARGE => KeywordSize::XXXLarge,
            structs::NS_STYLE_FONT_SIZE_NO_KEYWORD => {
                return FontSize {
                    size,
                    keyword_info: None,
                }
            }
            _ => unreachable!("mFontSizeKeyword should be an absolute keyword or NO_KEYWORD")
        };
        FontSize {
            size,
            keyword_info: Some(KeywordInfo {
                kw,
                factor: self.gecko.mFontSizeFactor,
                offset: Au(self.gecko.mFontSizeOffset).into()
            })
        }
    }

    pub fn set_font_weight(&mut self, v: longhands::font_weight::computed_value::T) {
        unsafe { bindings::Gecko_FontWeight_SetFloat(&mut self.gecko.mFont.weight, v.0) };
    }
    ${impl_simple_copy('font_weight', 'mFont.weight')}

    pub fn clone_font_weight(&self) -> longhands::font_weight::computed_value::T {
        let weight: f32 = unsafe {
            bindings::Gecko_FontWeight_ToFloat(self.gecko.mFont.weight)
        };
        longhands::font_weight::computed_value::T(weight)
    }

    pub fn set_font_stretch(&mut self, v: longhands::font_stretch::computed_value::T) {
        unsafe {
            bindings::Gecko_FontStretch_SetFloat(
                &mut self.gecko.mFont.stretch,
                v.value(),
            )
        };
    }
    ${impl_simple_copy('font_stretch', 'mFont.stretch')}
    pub fn clone_font_stretch(&self) -> longhands::font_stretch::computed_value::T {
        use crate::values::computed::font::FontStretch;
        use crate::values::computed::Percentage;
        use crate::values::generics::NonNegative;

        let stretch =
            unsafe { bindings::Gecko_FontStretch_ToFloat(self.gecko.mFont.stretch) };
        debug_assert!(stretch >= 0.);

        FontStretch(NonNegative(Percentage(stretch)))
    }

    pub fn set_font_style(&mut self, v: longhands::font_style::computed_value::T) {
        use crate::values::generics::font::FontStyle;
        let s = &mut self.gecko.mFont.style;
        unsafe {
            match v {
                FontStyle::Normal => bindings::Gecko_FontSlantStyle_SetNormal(s),
                FontStyle::Italic => bindings::Gecko_FontSlantStyle_SetItalic(s),
                FontStyle::Oblique(ref angle) => {
                    bindings::Gecko_FontSlantStyle_SetOblique(s, angle.0.degrees())
                }
            }
        }
    }
    ${impl_simple_copy('font_style', 'mFont.style')}
    pub fn clone_font_style(&self) -> longhands::font_style::computed_value::T {
        use crate::values::computed::font::FontStyle;
        FontStyle::from_gecko(self.gecko.mFont.style)
    }

    ${impl_simple_type_with_conversion("font_synthesis", "mFont.synthesis")}

    pub fn set_font_size_adjust(&mut self, v: longhands::font_size_adjust::computed_value::T) {
        use crate::properties::longhands::font_size_adjust::computed_value::T;
        match v {
            T::None => self.gecko.mFont.sizeAdjust = -1.0 as f32,
            T::Number(n) => self.gecko.mFont.sizeAdjust = n,
        }
    }

    pub fn copy_font_size_adjust_from(&mut self, other: &Self) {
        self.gecko.mFont.sizeAdjust = other.gecko.mFont.sizeAdjust;
    }

    pub fn reset_font_size_adjust(&mut self, other: &Self) {
        self.copy_font_size_adjust_from(other)
    }

    pub fn clone_font_size_adjust(&self) -> longhands::font_size_adjust::computed_value::T {
        use crate::properties::longhands::font_size_adjust::computed_value::T;
        T::from_gecko_adjust(self.gecko.mFont.sizeAdjust)
    }

    #[allow(non_snake_case)]
    pub fn set__x_lang(&mut self, v: longhands::_x_lang::computed_value::T) {
        let ptr = v.0.as_ptr();
        forget(v);
        unsafe {
            Gecko_nsStyleFont_SetLang(&mut *self.gecko, ptr);
        }
    }

    #[allow(non_snake_case)]
    pub fn copy__x_lang_from(&mut self, other: &Self) {
        unsafe {
            Gecko_nsStyleFont_CopyLangFrom(&mut *self.gecko, &*other.gecko);
        }
    }

    #[allow(non_snake_case)]
    pub fn reset__x_lang(&mut self, other: &Self) {
        self.copy__x_lang_from(other)
    }

    #[allow(non_snake_case)]
    pub fn clone__x_lang(&self) -> longhands::_x_lang::computed_value::T {
        longhands::_x_lang::computed_value::T(unsafe {
            Atom::from_raw(self.gecko.mLanguage.mRawPtr)
        })
    }

    #[allow(non_snake_case)]
    pub fn set__x_text_zoom(&mut self, v: longhands::_x_text_zoom::computed_value::T) {
        self.gecko.mAllowZoom = v.0;
    }

    #[allow(non_snake_case)]
    pub fn copy__x_text_zoom_from(&mut self, other: &Self) {
        self.gecko.mAllowZoom = other.gecko.mAllowZoom;
    }

    #[allow(non_snake_case)]
    pub fn reset__x_text_zoom(&mut self, other: &Self) {
        self.copy__x_text_zoom_from(other)
    }

    #[allow(non_snake_case)]
    pub fn clone__x_text_zoom(&self) -> longhands::_x_text_zoom::computed_value::T {
        longhands::_x_text_zoom::computed_value::T(self.gecko.mAllowZoom)
    }

    ${impl_simple("_moz_script_level", "mScriptLevel")}
    <% impl_simple_type_with_conversion("font_language_override", "mFont.languageOverride") %>

    pub fn set_font_variant_alternates(
        &mut self,
        v: values::computed::font::FontVariantAlternates,
    ) {
        use crate::gecko_bindings::bindings::{Gecko_ClearAlternateValues, Gecko_AppendAlternateValues};
        % for value in "normal swash stylistic ornaments annotation styleset character_variant historical".split():
            use crate::gecko_bindings::structs::NS_FONT_VARIANT_ALTERNATES_${value.upper()};
        % endfor
        use crate::values::specified::font::VariantAlternates;

        unsafe {
            Gecko_ClearAlternateValues(&mut self.gecko.mFont, v.len());
        }

        if v.0.is_empty() {
            self.gecko.mFont.variantAlternates = NS_FONT_VARIANT_ALTERNATES_NORMAL as u16;
            return;
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
        use crate::gecko_bindings::bindings::Gecko_CopyAlternateValuesFrom;

        self.gecko.mFont.variantAlternates = other.gecko.mFont.variantAlternates;
        unsafe {
            Gecko_CopyAlternateValuesFrom(&mut self.gecko.mFont, &other.gecko.mFont);
        }
    }

    pub fn reset_font_variant_alternates(&mut self, other: &Self) {
        self.copy_font_variant_alternates_from(other)
    }

    pub fn clone_font_variant_alternates(&self) -> values::computed::font::FontVariantAlternates {
        % for value in "normal swash stylistic ornaments annotation styleset character_variant historical".split():
            use crate::gecko_bindings::structs::NS_FONT_VARIANT_ALTERNATES_${value.upper()};
        % endfor
        use crate::values::specified::font::VariantAlternates;
        use crate::values::specified::font::VariantAlternatesList;
        use crate::values::CustomIdent;

        if self.gecko.mFont.variantAlternates == NS_FONT_VARIANT_ALTERNATES_NORMAL as u16 {
            return VariantAlternatesList(vec![].into_boxed_slice());
        }

        let mut alternates = Vec::with_capacity(self.gecko.mFont.alternateValues.len());
        if self.gecko.mFont.variantAlternates & (NS_FONT_VARIANT_ALTERNATES_HISTORICAL as u16) != 0 {
            alternates.push(VariantAlternates::HistoricalForms);
        }

        <%
            property_need_ident_list = "styleset character_variant".split()
        %>
        % for value in property_need_ident_list:
            let mut ${value}_list = Vec::new();
        % endfor

        for gecko_alternate_value in self.gecko.mFont.alternateValues.iter() {
            let ident = Atom::from(gecko_alternate_value.value.to_string());
            match gecko_alternate_value.alternate {
                % for value in "Swash Stylistic Ornaments Annotation".split():
                    NS_FONT_VARIANT_ALTERNATES_${value.upper()} => {
                        alternates.push(VariantAlternates::${value}(CustomIdent(ident)));
                    },
                % endfor
                % for value in property_need_ident_list:
                    NS_FONT_VARIANT_ALTERNATES_${value.upper()} => {
                        ${value}_list.push(CustomIdent(ident));
                    },
                % endfor
                _ => {
                    panic!("Found unexpected value for font-variant-alternates");
                }
            }
        }

        % for value in property_need_ident_list:
            if !${value}_list.is_empty() {
                alternates.push(VariantAlternates::${to_camel_case(value)}(${value}_list.into_boxed_slice()));
            }
        % endfor

        VariantAlternatesList(alternates.into_boxed_slice())
    }

    ${impl_simple_type_with_conversion("font_variant_ligatures", "mFont.variantLigatures")}
    ${impl_simple_type_with_conversion("font_variant_east_asian", "mFont.variantEastAsian")}
    ${impl_simple_type_with_conversion("font_variant_numeric", "mFont.variantNumeric")}

    #[allow(non_snake_case)]
    pub fn clone__moz_min_font_size_ratio(
        &self,
    ) -> longhands::_moz_min_font_size_ratio::computed_value::T {
        Percentage(self.gecko.mMinFontSizeRatio as f32 / 100.)
    }

    #[allow(non_snake_case)]
    pub fn set__moz_min_font_size_ratio(&mut self, v: longhands::_moz_min_font_size_ratio::computed_value::T) {
        let scaled = v.0 * 100.;
        let percentage = if scaled > 255. {
            255.
        } else if scaled < 0. {
            0.
        } else {
            scaled
        };

        self.gecko.mMinFontSizeRatio = percentage as u8;
    }

    ${impl_simple_copy('_moz_min_font_size_ratio', 'mMinFontSizeRatio')}
</%self:impl_trait>

<%def name="impl_copy_animation_or_transition_value(type, ident, gecko_ffi_name, member=None)">
    #[allow(non_snake_case)]
    pub fn copy_${type}_${ident}_from(&mut self, other: &Self) {
        self.gecko.m${type.capitalize()}s.ensure_len(other.gecko.m${type.capitalize()}s.len());

        let count = other.gecko.m${type.capitalize()}${gecko_ffi_name}Count;
        self.gecko.m${type.capitalize()}${gecko_ffi_name}Count = count;

        let iter = self.gecko.m${type.capitalize()}s.iter_mut().take(count as usize).zip(
            other.gecko.m${type.capitalize()}s.iter()
        );

        for (ours, others) in iter {
            % if member:
            ours.m${gecko_ffi_name}.${member} = others.m${gecko_ffi_name}.${member};
            % else:
            ours.m${gecko_ffi_name} = others.m${gecko_ffi_name};
            % endif
        }
    }

    #[allow(non_snake_case)]
    pub fn reset_${type}_${ident}(&mut self, other: &Self) {
        self.copy_${type}_${ident}_from(other)
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
        debug_assert_ne!(v.len(), 0);
        let input_len = v.len();
        self.gecko.m${type.capitalize()}s.ensure_len(input_len);

        self.gecko.m${type.capitalize()}${gecko_ffi_name}Count = input_len as u32;
        for (gecko, servo) in self.gecko.m${type.capitalize()}s.iter_mut().take(input_len as usize).zip(v) {
            gecko.m${gecko_ffi_name} = servo.seconds() * 1000.;
        }
    }
    #[allow(non_snake_case)]
    pub fn ${type}_${ident}_at(&self, index: usize)
        -> longhands::${type}_${ident}::computed_value::SingleComputedValue {
        use crate::values::computed::Time;
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
        debug_assert_ne!(v.len(), 0);
        let input_len = v.len();
        self.gecko.m${type.capitalize()}s.ensure_len(input_len);

        self.gecko.m${type.capitalize()}TimingFunctionCount = input_len as u32;
        for (gecko, servo) in self.gecko.m${type.capitalize()}s.iter_mut().take(input_len as usize).zip(v) {
            gecko.mTimingFunction.mTiming = servo;
        }
    }
    ${impl_animation_or_transition_count(type, 'timing_function', 'TimingFunction')}
    ${impl_copy_animation_or_transition_value(type, 'timing_function', "TimingFunction", "mTiming")}
    pub fn ${type}_timing_function_at(&self, index: usize)
        -> longhands::${type}_timing_function::computed_value::SingleComputedValue {
        self.gecko.m${type.capitalize()}s[index].mTimingFunction.mTiming
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
        use crate::properties::longhands::animation_${ident}::single_value::computed_value::T as Keyword;

        let v = v.into_iter();

        debug_assert_ne!(v.len(), 0);
        let input_len = v.len();
        self.gecko.mAnimations.ensure_len(input_len);

        self.gecko.mAnimation${gecko_ffi_name}Count = input_len as u32;

        for (gecko, servo) in self.gecko.mAnimations.iter_mut().take(input_len as usize).zip(v) {
            let result = match servo {
                % for value in keyword.gecko_values():
                    Keyword::${to_camel_case(value)} =>
                        structs::${keyword.gecko_constant(value)} ${keyword.maybe_cast(cast_type)},
                % endfor
            };
            gecko.m${gecko_ffi_name} = result;
        }
    }
    #[allow(non_snake_case)]
    pub fn animation_${ident}_at(&self, index: usize)
        -> longhands::animation_${ident}::computed_value::SingleComputedValue {
        use crate::properties::longhands::animation_${ident}::single_value::computed_value::T as Keyword;
        match self.gecko.mAnimations[index].m${gecko_ffi_name} ${keyword.maybe_cast("u32")} {
            % for value in keyword.gecko_values():
                structs::${keyword.gecko_constant(value)} => Keyword::${to_camel_case(value)},
            % endfor
            % if keyword.gecko_inexhaustive:
            _ => panic!("Found unexpected value for animation-${ident}"),
            % endif
        }
    }
    ${impl_animation_count(ident, gecko_ffi_name)}
    ${impl_copy_animation_value(ident, gecko_ffi_name)}
</%def>

<% skip_box_longhands= """display
                          animation-name animation-delay animation-duration
                          animation-direction animation-fill-mode animation-play-state
                          animation-iteration-count animation-timing-function
                          clear transition-duration transition-delay
                          transition-timing-function transition-property
                          transform-style shape-outside -webkit-line-clamp""" %>
<%self:impl_trait style_struct_name="Box" skip_longhands="${skip_box_longhands}">
    #[inline]
    pub fn set_display(&mut self, v: longhands::display::computed_value::T) {
        self.gecko.mDisplay = v;
        self.gecko.mOriginalDisplay = v;
    }

    #[inline]
    pub fn copy_display_from(&mut self, other: &Self) {
        self.gecko.mDisplay = other.gecko.mDisplay;
        self.gecko.mOriginalDisplay = other.gecko.mDisplay;
    }

    #[inline]
    pub fn reset_display(&mut self, other: &Self) {
        self.copy_display_from(other)
    }

    #[inline]
    pub fn set_adjusted_display(
        &mut self,
        v: longhands::display::computed_value::T,
        _is_item_or_root: bool
    ) {
        self.gecko.mDisplay = v;
    }

    #[inline]
    pub fn clone_display(&self) -> longhands::display::computed_value::T {
        self.gecko.mDisplay
    }

    <% clear_keyword = Keyword(
        "clear",
        "Left Right None Both",
        gecko_enum_prefix="StyleClear",
        gecko_inexhaustive=True,
    ) %>
    ${impl_keyword('clear', 'mBreakType', clear_keyword)}

    ${impl_transition_time_value('delay', 'Delay')}
    ${impl_transition_time_value('duration', 'Duration')}
    ${impl_transition_timing_function()}

    pub fn transition_combined_duration_at(&self, index: usize) -> f32 {
        // https://drafts.csswg.org/css-transitions/#transition-combined-duration
        self.gecko.mTransitions[index % self.gecko.mTransitionDurationCount as usize].mDuration.max(0.0)
            + self.gecko.mTransitions[index % self.gecko.mTransitionDelayCount as usize].mDelay
    }

    pub fn set_transition_property<I>(&mut self, v: I)
        where I: IntoIterator<Item = longhands::transition_property::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        use crate::gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_no_properties;
        use crate::gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_variable;
        use crate::gecko_bindings::structs::nsCSSPropertyID::eCSSProperty_UNKNOWN;

        let v = v.into_iter();

        if v.len() != 0 {
            self.gecko.mTransitions.ensure_len(v.len());
            self.gecko.mTransitionPropertyCount = v.len() as u32;
            for (servo, gecko) in v.zip(self.gecko.mTransitions.iter_mut()) {
                unsafe { gecko.mUnknownProperty.clear() };

                match servo {
                    TransitionProperty::Unsupported(ident) => {
                        gecko.mProperty = eCSSProperty_UNKNOWN;
                        gecko.mUnknownProperty.mRawPtr = ident.0.into_addrefed();
                    },
                    TransitionProperty::Custom(name) => {
                        gecko.mProperty = eCSSPropertyExtra_variable;
                        gecko.mUnknownProperty.mRawPtr = name.into_addrefed();
                    }
                    _ => gecko.mProperty = servo.to_nscsspropertyid().unwrap(),
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
        use crate::gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_all_properties;
        if self.gecko.mTransitionPropertyCount == 1 &&
            self.gecko.mTransitions[0].mProperty == eCSSPropertyExtra_all_properties &&
            self.transition_combined_duration_at(0) <= 0.0f32 {
            return false;
        }

        self.gecko.mTransitionPropertyCount > 0
    }

    pub fn transition_property_at(&self, index: usize)
        -> longhands::transition_property::computed_value::SingleComputedValue {
        use crate::gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_no_properties;
        use crate::gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_variable;
        use crate::gecko_bindings::structs::nsCSSPropertyID::eCSSProperty_UNKNOWN;

        let property = self.gecko.mTransitions[index].mProperty;
        if property == eCSSProperty_UNKNOWN {
            let atom = self.gecko.mTransitions[index].mUnknownProperty.mRawPtr;
            debug_assert!(!atom.is_null());
            TransitionProperty::Unsupported(CustomIdent(unsafe{
                Atom::from_raw(atom)
            }))
        } else if property == eCSSPropertyExtra_variable {
            let atom = self.gecko.mTransitions[index].mUnknownProperty.mRawPtr;
            debug_assert!(!atom.is_null());
            TransitionProperty::Custom(unsafe{
                Atom::from_raw(atom)
            })
        } else if property == eCSSPropertyExtra_no_properties {
            // Actually, we don't expect TransitionProperty::Unsupported also
            // represents "none", but if the caller wants to convert it, it is
            // fine. Please use it carefully.
            //
            // FIXME(emilio): This is a hack, is this reachable?
            TransitionProperty::Unsupported(CustomIdent(atom!("none")))
        } else {
            property.into()
        }
    }

    pub fn transition_nscsspropertyid_at(&self, index: usize) -> nsCSSPropertyID {
        self.gecko.mTransitions[index].mProperty
    }

    pub fn copy_transition_property_from(&mut self, other: &Self) {
        use crate::gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_variable;
        use crate::gecko_bindings::structs::nsCSSPropertyID::eCSSProperty_UNKNOWN;
        self.gecko.mTransitions.ensure_len(other.gecko.mTransitions.len());

        let count = other.gecko.mTransitionPropertyCount;
        self.gecko.mTransitionPropertyCount = count;

        for (index, transition) in self.gecko.mTransitions.iter_mut().enumerate().take(count as usize) {
            transition.mProperty = other.gecko.mTransitions[index].mProperty;
            unsafe { transition.mUnknownProperty.clear() };
            if transition.mProperty == eCSSProperty_UNKNOWN ||
               transition.mProperty == eCSSPropertyExtra_variable {
                let atom = other.gecko.mTransitions[index].mUnknownProperty.mRawPtr;
                debug_assert!(!atom.is_null());
                transition.mUnknownProperty.mRawPtr = unsafe { Atom::from_raw(atom) }.into_addrefed();
            }
        }
    }

    pub fn reset_transition_property(&mut self, other: &Self) {
        self.copy_transition_property_from(other)
    }

    // Hand-written because the Mako helpers transform `Preserve3d` into `PRESERVE3D`.
    pub fn set_transform_style(&mut self, v: TransformStyle) {
        self.gecko.mTransformStyle = match v {
            TransformStyle::Flat => structs::NS_STYLE_TRANSFORM_STYLE_FLAT as u8,
            TransformStyle::Preserve3d => structs::NS_STYLE_TRANSFORM_STYLE_PRESERVE_3D as u8,
        };
    }

    // Hand-written because the Mako helpers transform `Preserve3d` into `PRESERVE3D`.
    pub fn clone_transform_style(&self) -> TransformStyle {
        match self.gecko.mTransformStyle as u32 {
            structs::NS_STYLE_TRANSFORM_STYLE_FLAT => TransformStyle::Flat,
            structs::NS_STYLE_TRANSFORM_STYLE_PRESERVE_3D => TransformStyle::Preserve3d,
            _ => panic!("illegal transform style"),
        }
    }

    ${impl_simple_copy('transform_style', 'mTransformStyle')}

    ${impl_transition_count('property', 'Property')}

    pub fn animations_equals(&self, other: &Self) -> bool {
        return self.gecko.mAnimationNameCount == other.gecko.mAnimationNameCount
            && self.gecko.mAnimationDelayCount == other.gecko.mAnimationDelayCount
            && self.gecko.mAnimationDirectionCount == other.gecko.mAnimationDirectionCount
            && self.gecko.mAnimationDurationCount == other.gecko.mAnimationDurationCount
            && self.gecko.mAnimationFillModeCount == other.gecko.mAnimationFillModeCount
            && self.gecko.mAnimationIterationCountCount == other.gecko.mAnimationIterationCountCount
            && self.gecko.mAnimationPlayStateCount == other.gecko.mAnimationPlayStateCount
            && self.gecko.mAnimationTimingFunctionCount == other.gecko.mAnimationTimingFunctionCount
            && unsafe { bindings::Gecko_StyleAnimationsEquals(&self.gecko.mAnimations, &other.gecko.mAnimations) }
    }

    pub fn set_animation_name<I>(&mut self, v: I)
        where I: IntoIterator<Item = longhands::animation_name::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        let v = v.into_iter();
        debug_assert_ne!(v.len(), 0);
        self.gecko.mAnimations.ensure_len(v.len());

        self.gecko.mAnimationNameCount = v.len() as u32;
        for (servo, gecko) in v.zip(self.gecko.mAnimations.iter_mut()) {
            let atom = match servo.0 {
                None => atom!(""),
                Some(ref name) => name.as_atom().clone(),
            };
            unsafe { bindings::Gecko_SetAnimationName(gecko, atom.into_addrefed()); }
        }
    }
    pub fn animation_name_at(&self, index: usize)
        -> longhands::animation_name::computed_value::SingleComputedValue {
        use crate::properties::longhands::animation_name::single_value::SpecifiedValue as AnimationName;

        let atom = self.gecko.mAnimations[index].mName.mRawPtr;
        if atom == atom!("").as_ptr() {
            return AnimationName(None)
        }
        AnimationName(Some(KeyframesName::from_atom(unsafe { Atom::from_raw(atom) })))
    }
    pub fn copy_animation_name_from(&mut self, other: &Self) {
        self.gecko.mAnimationNameCount = other.gecko.mAnimationNameCount;
        unsafe { bindings::Gecko_CopyAnimationNames(&mut self.gecko.mAnimations, &other.gecko.mAnimations); }
    }

    pub fn reset_animation_name(&mut self, other: &Self) {
        self.copy_animation_name_from(other)
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
    where
        I: IntoIterator<Item = values::computed::AnimationIterationCount>,
        I::IntoIter: ExactSizeIterator + Clone
    {
        use std::f32;
        use crate::values::generics::box_::AnimationIterationCount;

        let v = v.into_iter();

        debug_assert_ne!(v.len(), 0);
        let input_len = v.len();
        self.gecko.mAnimations.ensure_len(input_len);

        self.gecko.mAnimationIterationCountCount = input_len as u32;
        for (gecko, servo) in self.gecko.mAnimations.iter_mut().take(input_len as usize).zip(v) {
            match servo {
                AnimationIterationCount::Number(n) => gecko.mIterationCount = n,
                AnimationIterationCount::Infinite => gecko.mIterationCount = f32::INFINITY,
            }
        }
    }

    pub fn animation_iteration_count_at(
        &self,
        index: usize,
    ) -> values::computed::AnimationIterationCount {
        use crate::values::generics::box_::AnimationIterationCount;

        if self.gecko.mAnimations[index].mIterationCount.is_infinite() {
            AnimationIterationCount::Infinite
        } else {
            AnimationIterationCount::Number(self.gecko.mAnimations[index].mIterationCount)
        }
    }

    ${impl_animation_count('iteration_count', 'IterationCount')}
    ${impl_copy_animation_value('iteration_count', 'IterationCount')}

    ${impl_animation_timing_function()}

    <% impl_shape_source("shape_outside", "mShapeOutside") %>

    #[allow(non_snake_case)]
    pub fn set__webkit_line_clamp(&mut self, v: longhands::_webkit_line_clamp::computed_value::T) {
        self.gecko.mLineClamp = match v {
            Either::First(n) => n.0 as u32,
            Either::Second(None_) => 0,
        };
    }

    ${impl_simple_copy('_webkit_line_clamp', 'mLineClamp')}

    #[allow(non_snake_case)]
    pub fn clone__webkit_line_clamp(&self) -> longhands::_webkit_line_clamp::computed_value::T {
        match self.gecko.mLineClamp {
            0 => Either::Second(None_),
            n => {
                debug_assert!(n <= std::i32::MAX as u32);
                Either::First((n as i32).into())
            }
        }
    }

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
        use crate::gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;
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
        use crate::gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        let count = other.gecko.${layers_field_name}.${field_name}Count;
        unsafe {
            Gecko_EnsureImageLayersLength(&mut self.gecko.${layers_field_name},
                                          count as usize,
                                          LayerType::${shorthand.title()});
        }
        // FIXME(emilio): This may be bogus in the same way as bug 1426246.
        for (layer, other) in self.gecko.${layers_field_name}.mLayers.iter_mut()
                                  .zip(other.gecko.${layers_field_name}.mLayers.iter())
                                  .take(count as usize) {
            layer.${field_name} = other.${field_name};
        }
        self.gecko.${layers_field_name}.${field_name}Count = count;
    }

    pub fn reset_${shorthand}_${name}(&mut self, other: &Self) {
        self.copy_${shorthand}_${name}_from(other)
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
    where
        I: IntoIterator<Item=longhands::${ident}::computed_value::single_value::T>,
        I::IntoIter: ExactSizeIterator,
    {
        use crate::properties::longhands::${ident}::single_value::computed_value::T as Keyword;
        use crate::gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

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
                    Keyword::${to_camel_case(value)} =>
                        structs::${keyword.gecko_constant(value)} ${keyword.maybe_cast('u8')},
                    % endfor
                }
            };
        }
    }

    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use crate::properties::longhands::${ident}::single_value::computed_value::T as Keyword;

        % if keyword.needs_cast():
        % for value in keyword.values_for('gecko'):
        const ${keyword.casted_constant_name(value, "u8")} : u8 =
            structs::${keyword.gecko_constant(value)} as u8;
        % endfor
        % endif

        longhands::${ident}::computed_value::List(
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
                            => Keyword::${to_camel_case(value)},
                        % endfor
                        % if keyword.gecko_inexhaustive:
                        _ => panic!("Found unexpected value in style struct for ${ident} property"),
                        % endif
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
        use crate::values::specified::background::BackgroundRepeatKeyword;
        use crate::gecko_bindings::structs::nsStyleImageLayers_Repeat;
        use crate::gecko_bindings::structs::StyleImageLayerRepeat;

        fn to_ns(repeat: BackgroundRepeatKeyword) -> StyleImageLayerRepeat {
            match repeat {
                BackgroundRepeatKeyword::Repeat => StyleImageLayerRepeat::Repeat,
                BackgroundRepeatKeyword::Space => StyleImageLayerRepeat::Space,
                BackgroundRepeatKeyword::Round => StyleImageLayerRepeat::Round,
                BackgroundRepeatKeyword::NoRepeat => StyleImageLayerRepeat::NoRepeat,
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
        use crate::properties::longhands::${shorthand}_repeat::single_value::computed_value::T;
        use crate::values::specified::background::BackgroundRepeatKeyword;
        use crate::gecko_bindings::structs::StyleImageLayerRepeat;

        fn to_servo(repeat: StyleImageLayerRepeat) -> BackgroundRepeatKeyword {
            match repeat {
                StyleImageLayerRepeat::Repeat => BackgroundRepeatKeyword::Repeat,
                StyleImageLayerRepeat::Space => BackgroundRepeatKeyword::Space,
                StyleImageLayerRepeat::Round => BackgroundRepeatKeyword::Round,
                StyleImageLayerRepeat::NoRepeat => BackgroundRepeatKeyword::NoRepeat,
                _ => panic!("Found unexpected value in style struct for ${shorthand}_repeat property"),
            }
        }

        longhands::${shorthand}_repeat::computed_value::List(
            self.gecko.${image_layers_field}.mLayers.iter()
                .take(self.gecko.${image_layers_field}.mRepeatCount as usize)
                .map(|ref layer| {
                    T(to_servo(layer.mRepeat.mXRepeat), to_servo(layer.mRepeat.mYRepeat))
                }).collect()
        )
    }

    <% impl_simple_image_array_property("clip", shorthand, image_layers_field, "mClip", struct_name) %>
    <% impl_simple_image_array_property("origin", shorthand, image_layers_field, "mOrigin", struct_name) %>

    % for (orientation, keyword) in [("x", "horizontal"), ("y", "vertical")]:
    pub fn copy_${shorthand}_position_${orientation}_from(&mut self, other: &Self) {
        use crate::gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        let count = other.gecko.${image_layers_field}.mPosition${orientation.upper()}Count;

        unsafe {
            Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field},
                                          count as usize,
                                          LayerType::${shorthand.capitalize()});
        }

        for (layer, other) in self.gecko.${image_layers_field}.mLayers.iter_mut()
                                  .zip(other.gecko.${image_layers_field}.mLayers.iter())
                                  .take(count as usize) {
            layer.mPosition.${keyword} = other.mPosition.${keyword};
        }
        self.gecko.${image_layers_field}.mPosition${orientation.upper()}Count = count;
    }

    pub fn reset_${shorthand}_position_${orientation}(&mut self, other: &Self) {
        self.copy_${shorthand}_position_${orientation}_from(other)
    }

    pub fn clone_${shorthand}_position_${orientation}(&self)
        -> longhands::${shorthand}_position_${orientation}::computed_value::T {
        longhands::${shorthand}_position_${orientation}::computed_value::List(
            self.gecko.${image_layers_field}.mLayers.iter()
                .take(self.gecko.${image_layers_field}.mPosition${orientation.upper()}Count as usize)
                .map(|position| position.mPosition.${keyword})
                .collect()
        )
    }

    pub fn set_${shorthand}_position_${orientation[0]}<I>(&mut self,
                                     v: I)
        where I: IntoIterator<Item = longhands::${shorthand}_position_${orientation[0]}
                                              ::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        use crate::gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        let v = v.into_iter();

        unsafe {
            Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field}, v.len(),
                                        LayerType::${shorthand.capitalize()});
        }

        self.gecko.${image_layers_field}.mPosition${orientation[0].upper()}Count = v.len() as u32;
        for (servo, geckolayer) in v.zip(self.gecko.${image_layers_field}
                                                           .mLayers.iter_mut()) {
            geckolayer.mPosition.${keyword} = servo;
        }
    }
    % endfor

    <%self:simple_image_array_property name="size" shorthand="${shorthand}" field_name="mSize">
        servo
    </%self:simple_image_array_property>

    pub fn clone_${shorthand}_size(&self) -> longhands::${shorthand}_size::computed_value::T {
        longhands::${shorthand}_size::computed_value::List(
            self.gecko.${image_layers_field}.mLayers.iter().map(|layer| layer.mSize).collect()
        )
    }

    pub fn copy_${shorthand}_image_from(&mut self, other: &Self) {
        use crate::gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;
        unsafe {
            let count = other.gecko.${image_layers_field}.mImageCount;
            Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field},
                                          count as usize,
                                          LayerType::${shorthand.capitalize()});

            for (layer, other) in self.gecko.${image_layers_field}.mLayers.iter_mut()
                                      .zip(other.gecko.${image_layers_field}.mLayers.iter())
                                      .take(count as usize) {
                Gecko_CopyImageValueFrom(&mut layer.mImage, &other.mImage);
            }
            self.gecko.${image_layers_field}.mImageCount = count;
        }
    }

    pub fn reset_${shorthand}_image(&mut self, other: &Self) {
        self.copy_${shorthand}_image_from(other)
    }

    #[allow(unused_variables)]
    pub fn set_${shorthand}_image<I>(&mut self, images: I)
        where I: IntoIterator<Item = longhands::${shorthand}_image::computed_value::single_value::T>,
              I::IntoIter: ExactSizeIterator
    {
        use crate::gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

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
            if let ImageLayer::Image(image) = image {
                geckoimage.mImage.set(image)
            }
        }
    }

    pub fn clone_${shorthand}_image(&self) -> longhands::${shorthand}_image::computed_value::T {
        longhands::${shorthand}_image::computed_value::List(
            self.gecko.${image_layers_field}.mLayers.iter()
                .take(self.gecko.${image_layers_field}.mImageCount as usize)
                .map(|ref layer| {
                    match unsafe { layer.mImage.into_image() } {
                        Some(image) => ImageLayer::Image(image),
                        None => ImageLayer::None,
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
        use crate::gecko_bindings::bindings::Gecko_FillAllImageLayers;
        use std::cmp;
        let mut max_len = 1;
        % for member in fill_fields.split():
            max_len = cmp::max(max_len, self.gecko.${image_layers_field}.${member}Count);
        % endfor
        unsafe {
            // While we could do this manually, we'd need to also manually
            // run all the copy constructors, so we just delegate to gecko
            Gecko_FillAllImageLayers(&mut self.gecko.${image_layers_field}, max_len);
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
                  skip_longhands="${skip_background_longhands}">

    <% impl_common_image_layer_properties("background") %>
    <% impl_simple_image_array_property("attachment", "background", "mImage", "mAttachment", "Background") %>
    <% impl_simple_image_array_property("blend_mode", "background", "mImage", "mBlendMode", "Background") %>
</%self:impl_trait>

<%self:impl_trait style_struct_name="List"
                  skip_longhands="list-style-image list-style-type -moz-image-region">

    pub fn set_list_style_image(&mut self, image: longhands::list_style_image::computed_value::T) {
        match image {
            UrlOrNone::None => {
                unsafe {
                    Gecko_SetListStyleImageNone(&mut *self.gecko);
                }
            }
            UrlOrNone::Url(ref url) => {
                unsafe {
                    Gecko_SetListStyleImageImageValue(&mut *self.gecko, url);
                }
            }
        }
    }

    pub fn copy_list_style_image_from(&mut self, other: &Self) {
        unsafe { Gecko_CopyListStyleImageFrom(&mut *self.gecko, &*other.gecko); }
    }

    pub fn reset_list_style_image(&mut self, other: &Self) {
        self.copy_list_style_image_from(other)
    }

    pub fn clone_list_style_image(&self) -> longhands::list_style_image::computed_value::T {
        if self.gecko.mListStyleImage.mRawPtr.is_null() {
            return UrlOrNone::None;
        }

        unsafe {
            let ref gecko_image_request = *self.gecko.mListStyleImage.mRawPtr;
            UrlOrNone::Url(ComputedImageUrl::from_image_request(gecko_image_request))
        }
    }

    pub fn set_list_style_type(&mut self, v: longhands::list_style_type::computed_value::T) {
        use crate::gecko_bindings::bindings::Gecko_SetCounterStyleToString;
        use nsstring::{nsACString, nsCStr};
        use self::longhands::list_style_type::computed_value::T;
        match v {
            T::CounterStyle(s) => s.to_gecko_value(&mut self.gecko.mCounterStyle),
            T::String(s) => unsafe {
                Gecko_SetCounterStyleToString(&mut self.gecko.mCounterStyle,
                                              &nsCStr::from(&s) as &nsACString)
            }
        }
    }

    pub fn copy_list_style_type_from(&mut self, other: &Self) {
        unsafe {
            Gecko_CopyCounterStyle(&mut self.gecko.mCounterStyle, &other.gecko.mCounterStyle);
        }
    }

    pub fn reset_list_style_type(&mut self, other: &Self) {
        self.copy_list_style_type_from(other)
    }

    pub fn clone_list_style_type(&self) -> longhands::list_style_type::computed_value::T {
        use self::longhands::list_style_type::computed_value::T;
        use crate::values::Either;
        use crate::values::generics::CounterStyleOrNone;

        let result = CounterStyleOrNone::from_gecko_value(&self.gecko.mCounterStyle);
        match result {
            Either::First(counter_style) => T::CounterStyle(counter_style),
            Either::Second(string) => T::String(string),
        }
    }

    #[allow(non_snake_case)]
    pub fn set__moz_image_region(&mut self, v: longhands::_moz_image_region::computed_value::T) {
        use crate::values::Either;
        use crate::values::generics::length::LengthPercentageOrAuto::*;

        match v {
            Either::Second(_auto) => {
                self.gecko.mImageRegion.x = 0;
                self.gecko.mImageRegion.y = 0;
                self.gecko.mImageRegion.width = 0;
                self.gecko.mImageRegion.height = 0;
            }
            Either::First(rect) => {
                self.gecko.mImageRegion.x = match rect.left {
                    LengthPercentage(v) => v.to_i32_au(),
                    Auto => 0,
                };
                self.gecko.mImageRegion.y = match rect.top {
                    LengthPercentage(v) => v.to_i32_au(),
                    Auto => 0,
                };
                self.gecko.mImageRegion.height = match rect.bottom {
                    LengthPercentage(value) => (Au::from(value) - Au(self.gecko.mImageRegion.y)).0,
                    Auto => 0,
                };
                self.gecko.mImageRegion.width = match rect.right {
                    LengthPercentage(value) => (Au::from(value) - Au(self.gecko.mImageRegion.x)).0,
                    Auto => 0,
                };
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn clone__moz_image_region(&self) -> longhands::_moz_image_region::computed_value::T {
        use crate::values::{Auto, Either};
        use crate::values::generics::length::LengthPercentageOrAuto::*;
        use crate::values::computed::ClipRect;

        // There is no ideal way to detect auto type for structs::nsRect and its components, so
        // if all components are zero, we use Auto.
        if self.gecko.mImageRegion.x == 0 &&
           self.gecko.mImageRegion.y == 0 &&
           self.gecko.mImageRegion.width == 0 &&
           self.gecko.mImageRegion.height == 0 {
           return Either::Second(Auto);
        }

        Either::First(ClipRect {
            top: LengthPercentage(Au(self.gecko.mImageRegion.y).into()),
            right: LengthPercentage(Au(self.gecko.mImageRegion.width + self.gecko.mImageRegion.x).into()),
            bottom: LengthPercentage(Au(self.gecko.mImageRegion.height + self.gecko.mImageRegion.y).into()),
            left: LengthPercentage(Au(self.gecko.mImageRegion.x).into()),
        })
    }

    ${impl_simple_copy('_moz_image_region', 'mImageRegion')}

</%self:impl_trait>

<%self:impl_trait style_struct_name="Table" skip_longhands="-x-span">
    #[allow(non_snake_case)]
    pub fn set__x_span(&mut self, v: longhands::_x_span::computed_value::T) {
        self.gecko.mSpan = v.0
    }

    #[allow(non_snake_case)]
    pub fn clone__x_span(&self) -> longhands::_x_span::computed_value::T {
        longhands::_x_span::computed_value::T(
            self.gecko.mSpan
        )
    }

    ${impl_simple_copy('_x_span', 'mSpan')}
</%self:impl_trait>

<%self:impl_trait style_struct_name="Effects" skip_longhands="clip">
    pub fn set_clip(&mut self, v: longhands::clip::computed_value::T) {
        use crate::gecko_bindings::structs::NS_STYLE_CLIP_AUTO;
        use crate::gecko_bindings::structs::NS_STYLE_CLIP_RECT;
        use crate::gecko_bindings::structs::NS_STYLE_CLIP_LEFT_AUTO;
        use crate::gecko_bindings::structs::NS_STYLE_CLIP_TOP_AUTO;
        use crate::gecko_bindings::structs::NS_STYLE_CLIP_RIGHT_AUTO;
        use crate::gecko_bindings::structs::NS_STYLE_CLIP_BOTTOM_AUTO;
        use crate::values::generics::length::LengthPercentageOrAuto::*;
        use crate::values::Either;

        match v {
            Either::First(rect) => {
                self.gecko.mClipFlags = NS_STYLE_CLIP_RECT as u8;
                self.gecko.mClip.x = match rect.left {
                    LengthPercentage(l) => l.to_i32_au(),
                    Auto => {
                        self.gecko.mClipFlags |= NS_STYLE_CLIP_LEFT_AUTO as u8;
                        0
                    }
                };

                self.gecko.mClip.y = match rect.top {
                    LengthPercentage(l) => l.to_i32_au(),
                    Auto => {
                        self.gecko.mClipFlags |= NS_STYLE_CLIP_TOP_AUTO as u8;
                        0
                    }
                };

                self.gecko.mClip.height = match rect.bottom {
                    LengthPercentage(l) => (Au::from(l) - Au(self.gecko.mClip.y)).0,
                    Auto => {
                        self.gecko.mClipFlags |= NS_STYLE_CLIP_BOTTOM_AUTO as u8;
                        1 << 30 // NS_MAXSIZE
                    }
                };

                self.gecko.mClip.width = match rect.right {
                    LengthPercentage(l) => (Au::from(l) - Au(self.gecko.mClip.x)).0,
                    Auto => {
                        self.gecko.mClipFlags |= NS_STYLE_CLIP_RIGHT_AUTO as u8;
                        1 << 30 // NS_MAXSIZE
                    }
                };
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

    pub fn reset_clip(&mut self, other: &Self) {
        self.copy_clip_from(other)
    }

    pub fn clone_clip(&self) -> longhands::clip::computed_value::T {
        use crate::gecko_bindings::structs::NS_STYLE_CLIP_AUTO;
        use crate::gecko_bindings::structs::NS_STYLE_CLIP_BOTTOM_AUTO;
        use crate::gecko_bindings::structs::NS_STYLE_CLIP_LEFT_AUTO;
        use crate::gecko_bindings::structs::NS_STYLE_CLIP_RIGHT_AUTO;
        use crate::gecko_bindings::structs::NS_STYLE_CLIP_TOP_AUTO;
        use crate::values::generics::length::LengthPercentageOrAuto::*;
        use crate::values::computed::{ClipRect, ClipRectOrAuto};
        use crate::values::Either;

        if self.gecko.mClipFlags == NS_STYLE_CLIP_AUTO as u8 {
            return ClipRectOrAuto::auto()
        }
        let left = if self.gecko.mClipFlags & NS_STYLE_CLIP_LEFT_AUTO as u8 != 0 {
            debug_assert_eq!(self.gecko.mClip.x, 0);
            Auto
        } else {
            LengthPercentage(Au(self.gecko.mClip.x).into())
        };

        let top = if self.gecko.mClipFlags & NS_STYLE_CLIP_TOP_AUTO as u8 != 0 {
            debug_assert_eq!(self.gecko.mClip.y, 0);
            Auto
        } else {
            LengthPercentage(Au(self.gecko.mClip.y).into())
        };

        let bottom = if self.gecko.mClipFlags & NS_STYLE_CLIP_BOTTOM_AUTO as u8 != 0 {
            debug_assert_eq!(self.gecko.mClip.height, 1 << 30); // NS_MAXSIZE
            Auto
        } else {
            LengthPercentage(Au(self.gecko.mClip.y + self.gecko.mClip.height).into())
        };

        let right = if self.gecko.mClipFlags & NS_STYLE_CLIP_RIGHT_AUTO as u8 != 0 {
            debug_assert_eq!(self.gecko.mClip.width, 1 << 30); // NS_MAXSIZE
            Auto
        } else {
            LengthPercentage(Au(self.gecko.mClip.x + self.gecko.mClip.width).into())
        };

        Either::First(ClipRect { top, right, bottom, left })
    }
</%self:impl_trait>

<%self:impl_trait style_struct_name="InheritedBox">
</%self:impl_trait>

<%self:impl_trait style_struct_name="InheritedTable"
                  skip_longhands="border-spacing">

    pub fn set_border_spacing(&mut self, v: longhands::border_spacing::computed_value::T) {
        self.gecko.mBorderSpacingCol = v.horizontal().0;
        self.gecko.mBorderSpacingRow = v.vertical().0;
    }

    pub fn copy_border_spacing_from(&mut self, other: &Self) {
        self.gecko.mBorderSpacingCol = other.gecko.mBorderSpacingCol;
        self.gecko.mBorderSpacingRow = other.gecko.mBorderSpacingRow;
    }

    pub fn reset_border_spacing(&mut self, other: &Self) {
        self.copy_border_spacing_from(other)
    }

    pub fn clone_border_spacing(&self) -> longhands::border_spacing::computed_value::T {
        longhands::border_spacing::computed_value::T::new(
            Au(self.gecko.mBorderSpacingCol).into(),
            Au(self.gecko.mBorderSpacingRow).into()
        )
    }
</%self:impl_trait>


<%self:impl_trait style_struct_name="InheritedText"
                  skip_longhands="text-align text-emphasis-style
                                  -webkit-text-stroke-width text-emphasis-position">

    <% text_align_keyword = Keyword("text-align",
                                    "start end left right center justify -moz-center -moz-left -moz-right char",
                                    gecko_strip_moz_prefix=False) %>
    ${impl_keyword('text_align', 'mTextAlign', text_align_keyword)}

    fn clear_text_emphasis_style_if_string(&mut self) {
        if self.gecko.mTextEmphasisStyle == structs::NS_STYLE_TEXT_EMPHASIS_STYLE_STRING as u8 {
            self.gecko.mTextEmphasisStyleString.truncate();
            self.gecko.mTextEmphasisStyle = structs::NS_STYLE_TEXT_EMPHASIS_STYLE_NONE as u8;
        }
    }

    ${impl_simple_type_with_conversion("text_emphasis_position")}

    pub fn set_text_emphasis_style(&mut self, v: values::computed::TextEmphasisStyle) {
        use crate::values::computed::TextEmphasisStyle;
        use crate::values::specified::text::{TextEmphasisFillMode, TextEmphasisShapeKeyword};

        self.clear_text_emphasis_style_if_string();
        let (te, s) = match v {
            TextEmphasisStyle::None => (structs::NS_STYLE_TEXT_EMPHASIS_STYLE_NONE, ""),
            TextEmphasisStyle::Keyword(ref keyword) => {
                let fill = match keyword.fill {
                    TextEmphasisFillMode::Filled => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_FILLED,
                    TextEmphasisFillMode::Open => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_OPEN,
                };
                let shape = match keyword.shape {
                    TextEmphasisShapeKeyword::Dot => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_DOT,
                    TextEmphasisShapeKeyword::Circle => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_CIRCLE,
                    TextEmphasisShapeKeyword::DoubleCircle => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_DOUBLE_CIRCLE,
                    TextEmphasisShapeKeyword::Triangle => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_TRIANGLE,
                    TextEmphasisShapeKeyword::Sesame => structs::NS_STYLE_TEXT_EMPHASIS_STYLE_SESAME,
                };

                (shape | fill, keyword.shape.char(keyword.fill))
            },
            TextEmphasisStyle::String(ref s) => {
                (structs::NS_STYLE_TEXT_EMPHASIS_STYLE_STRING, &**s)
            },
        };
        self.gecko.mTextEmphasisStyleString.assign_str(s);
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

    pub fn reset_text_emphasis_style(&mut self, other: &Self) {
        self.copy_text_emphasis_style_from(other)
    }

    pub fn clone_text_emphasis_style(&self) -> values::computed::TextEmphasisStyle {
        use crate::values::computed::TextEmphasisStyle;
        use crate::values::computed::text::TextEmphasisKeywordValue;
        use crate::values::specified::text::{TextEmphasisFillMode, TextEmphasisShapeKeyword};

        if self.gecko.mTextEmphasisStyle == structs::NS_STYLE_TEXT_EMPHASIS_STYLE_NONE as u8 {
            return TextEmphasisStyle::None;
        }

        if self.gecko.mTextEmphasisStyle == structs::NS_STYLE_TEXT_EMPHASIS_STYLE_STRING as u8 {
            return TextEmphasisStyle::String(self.gecko.mTextEmphasisStyleString.to_string());
        }

        let fill =
            self.gecko.mTextEmphasisStyle & structs::NS_STYLE_TEXT_EMPHASIS_STYLE_OPEN as u8 == 0;

        let fill = if fill { TextEmphasisFillMode::Filled } else { TextEmphasisFillMode::Open };

        let shape =
            match self.gecko.mTextEmphasisStyle as u32 & !structs::NS_STYLE_TEXT_EMPHASIS_STYLE_OPEN {
                structs::NS_STYLE_TEXT_EMPHASIS_STYLE_DOT => TextEmphasisShapeKeyword::Dot,
                structs::NS_STYLE_TEXT_EMPHASIS_STYLE_CIRCLE => TextEmphasisShapeKeyword::Circle,
                structs::NS_STYLE_TEXT_EMPHASIS_STYLE_DOUBLE_CIRCLE => TextEmphasisShapeKeyword::DoubleCircle,
                structs::NS_STYLE_TEXT_EMPHASIS_STYLE_TRIANGLE => TextEmphasisShapeKeyword::Triangle,
                structs::NS_STYLE_TEXT_EMPHASIS_STYLE_SESAME => TextEmphasisShapeKeyword::Sesame,
                _ => panic!("Unexpected value in style struct for text-emphasis-style property")
            };

        TextEmphasisStyle::Keyword(TextEmphasisKeywordValue { fill, shape })
    }

    ${impl_non_negative_length('_webkit_text_stroke_width',
                               'mWebkitTextStrokeWidth')}

</%self:impl_trait>

<%self:impl_trait style_struct_name="Text" skip_longhands="initial-letter">
    pub fn set_initial_letter(&mut self, v: longhands::initial_letter::computed_value::T) {
        use crate::values::generics::text::InitialLetter;
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

    pub fn reset_initial_letter(&mut self, other: &Self) {
        self.copy_initial_letter_from(other)
    }

    pub fn clone_initial_letter(&self) -> longhands::initial_letter::computed_value::T {
        use crate::values::generics::text::InitialLetter;

        if self.gecko.mInitialLetterSize == 0. && self.gecko.mInitialLetterSink == 0 {
            InitialLetter::Normal
        } else if self.gecko.mInitialLetterSize.floor() as i32 == self.gecko.mInitialLetterSink {
            InitialLetter::Specified(self.gecko.mInitialLetterSize, None)
        } else {
            InitialLetter::Specified(self.gecko.mInitialLetterSize, Some(self.gecko.mInitialLetterSink))
        }
    }
</%self:impl_trait>

// Set SVGPathData to StyleShapeSource.
fn set_style_svg_path(
    shape_source: &mut structs::mozilla::StyleShapeSource,
    servo_path: values::specified::svg_path::SVGPathData,
    fill: values::generics::basic_shape::FillRule,
) {
    // Setup path.
    unsafe {
        bindings::Gecko_SetToSVGPath(
            shape_source,
            servo_path.0.forget(),
            fill,
        );
    }
}

<%def name="impl_shape_source(ident, gecko_ffi_name)">
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        use crate::values::generics::basic_shape::ShapeSource;
        use crate::gecko_bindings::structs::StyleShapeSourceType;
        use crate::gecko_bindings::structs::StyleGeometryBox;

        let ref mut ${ident} = self.gecko.${gecko_ffi_name};

        // clean up existing struct.
        unsafe { bindings::Gecko_DestroyShapeSource(${ident}) };

        ${ident}.mType = StyleShapeSourceType::None;

        match v {
            ShapeSource::None => {} // don't change the type
            ShapeSource::ImageOrUrl(image) => {
                % if ident == "clip_path":
                use crate::values::generics::image::Image;

                let image = Image::Url(ComputedImageUrl(image));
                % endif
                unsafe {
                    bindings::Gecko_NewShapeImage(${ident});
                    let style_image = &mut *${ident}.__bindgen_anon_1.mShapeImage.as_mut().mPtr;
                    style_image.set(image);
                }
            }
            ShapeSource::Box(reference) => {
                ${ident}.mReferenceBox = reference.into();
                ${ident}.mType = StyleShapeSourceType::Box;
            }
            ShapeSource::Path(p) => set_style_svg_path(${ident}, p.path, p.fill),
            ShapeSource::Shape(servo_shape, maybe_box) => {
                unsafe {
                    ${ident}.__bindgen_anon_1.mBasicShape.as_mut().mPtr =
                        Box::into_raw(servo_shape);
                }
                ${ident}.mReferenceBox =
                    maybe_box.map(Into::into).unwrap_or(StyleGeometryBox::NoBox);
                ${ident}.mType = StyleShapeSourceType::Shape;
            }
        }

    }

    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        (&self.gecko.${gecko_ffi_name}).into()
    }

    pub fn copy_${ident}_from(&mut self, other: &Self) {
        use crate::gecko_bindings::bindings::Gecko_CopyShapeSourceFrom;
        unsafe {
            Gecko_CopyShapeSourceFrom(&mut self.gecko.${gecko_ffi_name}, &other.gecko.${gecko_ffi_name});
        }
    }

    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }
</%def>

<% skip_svg_longhands = """
mask-mode mask-repeat mask-clip mask-origin mask-composite mask-position-x mask-position-y mask-size mask-image
clip-path
"""
%>
<%self:impl_trait style_struct_name="SVG"
                  skip_longhands="${skip_svg_longhands}">

    <% impl_common_image_layer_properties("mask") %>
    <% impl_simple_image_array_property("mode", "mask", "mMask", "mMaskMode", "SVG") %>
    <% impl_simple_image_array_property("composite", "mask", "mMask", "mComposite", "SVG") %>
    <% impl_shape_source("clip_path", "mClipPath") %>
</%self:impl_trait>

<%self:impl_trait style_struct_name="InheritedSVG"
                  skip_longhands="paint-order stroke-dasharray">
    pub fn set_paint_order(&mut self, v: longhands::paint_order::computed_value::T) {
        self.gecko.mPaintOrder = v.0;
    }

    ${impl_simple_copy('paint_order', 'mPaintOrder')}

    pub fn clone_paint_order(&self) -> longhands::paint_order::computed_value::T {
        use crate::properties::longhands::paint_order::computed_value::T;
        T(self.gecko.mPaintOrder)
    }

    pub fn set_stroke_dasharray(&mut self, v: longhands::stroke_dasharray::computed_value::T) {
        use crate::gecko_bindings::structs::nsStyleSVG_STROKE_DASHARRAY_CONTEXT as CONTEXT_VALUE;
        use crate::values::generics::svg::SVGStrokeDashArray;

        match v {
            SVGStrokeDashArray::Values(v) => {
                let v = v.into_iter();
                self.gecko.mContextFlags &= !CONTEXT_VALUE;
                unsafe {
                    bindings::Gecko_nsStyleSVG_SetDashArrayLength(&mut *self.gecko, v.len() as u32);
                }
                for (gecko, servo) in self.gecko.mStrokeDasharray.iter_mut().zip(v) {
                    *gecko = servo;
                }
            }
            SVGStrokeDashArray::ContextValue => {
                self.gecko.mContextFlags |= CONTEXT_VALUE;
                unsafe {
                    bindings::Gecko_nsStyleSVG_SetDashArrayLength(&mut *self.gecko, 0);
                }
            }
        }
    }

    pub fn copy_stroke_dasharray_from(&mut self, other: &Self) {
        use crate::gecko_bindings::structs::nsStyleSVG_STROKE_DASHARRAY_CONTEXT as CONTEXT_VALUE;
        unsafe {
            bindings::Gecko_nsStyleSVG_CopyDashArray(&mut *self.gecko, &*other.gecko);
        }
        self.gecko.mContextFlags =
            (self.gecko.mContextFlags & !CONTEXT_VALUE) |
            (other.gecko.mContextFlags & CONTEXT_VALUE);
    }

    pub fn reset_stroke_dasharray(&mut self, other: &Self) {
        self.copy_stroke_dasharray_from(other)
    }

    pub fn clone_stroke_dasharray(&self) -> longhands::stroke_dasharray::computed_value::T {
        use crate::gecko_bindings::structs::nsStyleSVG_STROKE_DASHARRAY_CONTEXT as CONTEXT_VALUE;
        use crate::values::generics::svg::SVGStrokeDashArray;

        if self.gecko.mContextFlags & CONTEXT_VALUE != 0 {
            debug_assert_eq!(self.gecko.mStrokeDasharray.len(), 0);
            return SVGStrokeDashArray::ContextValue;
        }
        SVGStrokeDashArray::Values(self.gecko.mStrokeDasharray.iter().cloned().collect())
    }
</%self:impl_trait>

<%self:impl_trait style_struct_name="InheritedUI" skip_longhands="cursor">
    pub fn set_cursor(&mut self, v: longhands::cursor::computed_value::T) {
        self.gecko.mCursor = v.keyword;
        unsafe {
            Gecko_SetCursorArrayLength(&mut *self.gecko, v.images.len());
        }
        for i in 0..v.images.len() {
            unsafe {
                Gecko_SetCursorImageValue(
                    &mut self.gecko.mCursorImages[i],
                    &v.images[i].url
                );
            }

            match v.images[i].hotspot {
                Some((x, y)) => {
                    self.gecko.mCursorImages[i].mHaveHotspot = true;
                    self.gecko.mCursorImages[i].mHotspotX = x;
                    self.gecko.mCursorImages[i].mHotspotY = y;
                },
                _ => {
                    self.gecko.mCursorImages[i].mHaveHotspot = false;
                }
            }
        }
    }

    pub fn copy_cursor_from(&mut self, other: &Self) {
        self.gecko.mCursor = other.gecko.mCursor;
        unsafe {
            Gecko_CopyCursorArrayFrom(&mut *self.gecko, &*other.gecko);
        }
    }

    pub fn reset_cursor(&mut self, other: &Self) {
        self.copy_cursor_from(other)
    }

    pub fn clone_cursor(&self) -> longhands::cursor::computed_value::T {
        use crate::values::computed::ui::CursorImage;

        let keyword = self.gecko.mCursor;

        let images = self.gecko.mCursorImages.iter().map(|gecko_cursor_image| {
            let url = unsafe {
                let gecko_image_request = gecko_cursor_image.mImage.mRawPtr.as_ref().unwrap();
                ComputedImageUrl::from_image_request(&gecko_image_request)
            };

            let hotspot =
                if gecko_cursor_image.mHaveHotspot {
                    Some((gecko_cursor_image.mHotspotX, gecko_cursor_image.mHotspotY))
                } else {
                    None
                };

            CursorImage { url, hotspot }
        }).collect::<Vec<_>>().into_boxed_slice();

        longhands::cursor::computed_value::T { images, keyword }
    }
</%self:impl_trait>

<%self:impl_trait style_struct_name="Column"
                  skip_longhands="column-count column-rule-width column-rule-style">

    #[allow(unused_unsafe)]
    pub fn set_column_count(&mut self, v: longhands::column_count::computed_value::T) {
        use crate::gecko_bindings::structs::{nsStyleColumn_kColumnCountAuto, nsStyleColumn_kMaxColumnCount};

        self.gecko.mColumnCount = match v {
            ColumnCount::Integer(integer) => {
                cmp::min(integer.0 as u32, unsafe { nsStyleColumn_kMaxColumnCount })
            },
            ColumnCount::Auto => nsStyleColumn_kColumnCountAuto
        };
    }

    ${impl_simple_copy('column_count', 'mColumnCount')}

    pub fn clone_column_count(&self) -> longhands::column_count::computed_value::T {
        use crate::gecko_bindings::structs::{nsStyleColumn_kColumnCountAuto, nsStyleColumn_kMaxColumnCount};
        if self.gecko.mColumnCount != nsStyleColumn_kColumnCountAuto {
            debug_assert!(self.gecko.mColumnCount >= 1 &&
                          self.gecko.mColumnCount <= nsStyleColumn_kMaxColumnCount);
            ColumnCount::Integer((self.gecko.mColumnCount as i32).into())
        } else {
            ColumnCount::Auto
        }
    }

    <% impl_non_negative_length("column_rule_width", "mColumnRuleWidth",
                                round_to_pixels=True) %>
    ${impl_simple('column_rule_style', 'mColumnRuleStyle')}
</%self:impl_trait>

<%self:impl_trait style_struct_name="Counters"
                  skip_longhands="content counter-increment counter-reset counter-set">
    pub fn ineffective_content_property(&self) -> bool {
        self.gecko.mContents.is_empty()
    }

    pub fn set_content(&mut self, v: longhands::content::computed_value::T) {
        use crate::values::CustomIdent;
        use crate::values::generics::counters::{Content, ContentItem};
        use crate::values::generics::CounterStyleOrNone;
        use crate::gecko_bindings::structs::nsStyleContentData;
        use crate::gecko_bindings::structs::nsStyleContentAttr;
        use crate::gecko_bindings::structs::StyleContentType;
        use crate::gecko_bindings::bindings::Gecko_ClearAndResizeStyleContents;

        // Converts a string as utf16, and returns an owned, zero-terminated raw buffer.
        fn as_utf16_and_forget(s: &str) -> *mut u16 {
            use std::mem;
            let mut vec = s.encode_utf16().collect::<Vec<_>>();
            vec.push(0u16);
            let ptr = vec.as_mut_ptr();
            mem::forget(vec);
            ptr
        }

        fn set_counter_function(
            data: &mut nsStyleContentData,
            content_type: StyleContentType,
            name: CustomIdent,
            sep: &str,
            style: CounterStyleOrNone,
        ) {
            debug_assert!(content_type == StyleContentType::Counter ||
                          content_type == StyleContentType::Counters);
            let counter_func = unsafe {
                bindings::Gecko_SetCounterFunction(data, content_type).as_mut().unwrap()
            };
            counter_func.mIdent.set_move(unsafe {
                RefPtr::from_addrefed(name.0.into_addrefed())
            });
            if content_type == StyleContentType::Counters {
                counter_func.mSeparator.assign_str(sep);
            }
            style.to_gecko_value(&mut counter_func.mCounterStyle);
        }

        match v {
            Content::None |
            Content::Normal => {
                // Ensure destructors run, otherwise we could leak.
                if !self.gecko.mContents.is_empty() {
                    unsafe {
                        Gecko_ClearAndResizeStyleContents(&mut *self.gecko, 0);
                    }
                }
            },
            Content::MozAltContent => {
                unsafe {
                    Gecko_ClearAndResizeStyleContents(&mut *self.gecko, 1);
                    *self.gecko.mContents[0].mContent.mString.as_mut() = ptr::null_mut();
                }
                self.gecko.mContents[0].mType = StyleContentType::AltContent;
            },
            Content::Items(items) => {
                unsafe {
                    Gecko_ClearAndResizeStyleContents(&mut *self.gecko,
                                                      items.len() as u32);
                }
                for (i, item) in items.into_vec().into_iter().enumerate() {
                    // NB: Gecko compares the mString value if type is not image
                    // or URI independently of whatever gets there. In the quote
                    // cases, they set it to null, so do the same here.
                    unsafe {
                        *self.gecko.mContents[i].mContent.mString.as_mut() = ptr::null_mut();
                    }
                    match item {
                        ContentItem::String(ref value) => {
                            self.gecko.mContents[i].mType = StyleContentType::String;
                            unsafe {
                                // NB: we share allocators, so doing this is fine.
                                *self.gecko.mContents[i].mContent.mString.as_mut() =
                                    as_utf16_and_forget(&value);
                            }
                        }
                        ContentItem::Attr(ref attr) => {
                            self.gecko.mContents[i].mType = StyleContentType::Attr;
                            unsafe {
                                // NB: we share allocators, so doing this is fine.
                                let maybe_ns = attr.namespace.clone();
                                let attr_struct = Box::new(nsStyleContentAttr {
                                    mName: structs::RefPtr {
                                        mRawPtr: attr.attribute.clone().into_addrefed(),
                                        _phantom_0: PhantomData,
                                    },
                                    mNamespaceURL: structs::RefPtr {
                                        mRawPtr: maybe_ns.map_or(ptr::null_mut(), |x| (x.1).0.into_addrefed()),
                                        _phantom_0: PhantomData,
                                    },
                                });
                                *self.gecko.mContents[i].mContent.mAttr.as_mut() =
                                    Box::into_raw(attr_struct);
                            }
                        }
                        ContentItem::OpenQuote
                            => self.gecko.mContents[i].mType = StyleContentType::OpenQuote,
                        ContentItem::CloseQuote
                            => self.gecko.mContents[i].mType = StyleContentType::CloseQuote,
                        ContentItem::NoOpenQuote
                            => self.gecko.mContents[i].mType = StyleContentType::NoOpenQuote,
                        ContentItem::NoCloseQuote
                            => self.gecko.mContents[i].mType = StyleContentType::NoCloseQuote,
                        ContentItem::Counter(name, style) => {
                            set_counter_function(
                                &mut self.gecko.mContents[i],
                                StyleContentType::Counter,
                                name,
                                "",
                                style,
                            );
                        }
                        ContentItem::Counters(name, sep, style) => {
                            set_counter_function(
                                &mut self.gecko.mContents[i],
                                StyleContentType::Counters,
                                name,
                                &sep,
                                style,
                            );
                        }
                        ContentItem::Url(ref url) => {
                            unsafe {
                                bindings::Gecko_SetContentDataImageValue(
                                    &mut self.gecko.mContents[i],
                                    url,
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn copy_content_from(&mut self, other: &Self) {
        use crate::gecko_bindings::bindings::Gecko_CopyStyleContentsFrom;
        unsafe {
            Gecko_CopyStyleContentsFrom(&mut *self.gecko, &*other.gecko)
        }
    }

    pub fn reset_content(&mut self, other: &Self) {
        self.copy_content_from(other)
    }

    pub fn clone_content(&self) -> longhands::content::computed_value::T {
        use {Atom, Namespace};
        use crate::gecko::conversions::string_from_chars_pointer;
        use crate::gecko_bindings::structs::StyleContentType;
        use crate::values::generics::counters::{Content, ContentItem};
        use crate::values::{CustomIdent, Either};
        use crate::values::generics::CounterStyleOrNone;
        use crate::values::specified::Attr;

        if self.gecko.mContents.is_empty() {
            return Content::None;
        }

        if self.gecko.mContents.len() == 1 &&
           self.gecko.mContents[0].mType == StyleContentType::AltContent {
            return Content::MozAltContent;
        }

        Content::Items(
            self.gecko.mContents.iter().map(|gecko_content| {
                match gecko_content.mType {
                    StyleContentType::OpenQuote => ContentItem::OpenQuote,
                    StyleContentType::CloseQuote => ContentItem::CloseQuote,
                    StyleContentType::NoOpenQuote => ContentItem::NoOpenQuote,
                    StyleContentType::NoCloseQuote => ContentItem::NoCloseQuote,
                    StyleContentType::String => {
                        let gecko_chars = unsafe { gecko_content.mContent.mString.as_ref() };
                        let string = unsafe { string_from_chars_pointer(*gecko_chars) };
                        ContentItem::String(string.into_boxed_str())
                    },
                    StyleContentType::Attr => {
                        let (namespace, attribute) = unsafe {
                            let s = &**gecko_content.mContent.mAttr.as_ref();
                            let ns = if s.mNamespaceURL.mRawPtr.is_null() {
                                None
                            } else {
                                // FIXME(bholley): We don't have any way to get the prefix here. :-(
                                let prefix = atom!("");
                                Some((prefix, Namespace(Atom::from_raw(s.mNamespaceURL.mRawPtr))))
                            };
                            (ns, Atom::from_raw(s.mName.mRawPtr))
                        };
                        ContentItem::Attr(Attr { namespace, attribute })
                    },
                    StyleContentType::Counter | StyleContentType::Counters => {
                        let gecko_function =
                            unsafe { &**gecko_content.mContent.mCounters.as_ref() };
                        let ident = CustomIdent(unsafe {
                            Atom::from_raw(gecko_function.mIdent.mRawPtr)
                        });
                        let style =
                            CounterStyleOrNone::from_gecko_value(&gecko_function.mCounterStyle);
                        let style = match style {
                            Either::First(counter_style) => counter_style,
                            Either::Second(_) =>
                                unreachable!("counter function shouldn't have single string type"),
                        };
                        if gecko_content.mType == StyleContentType::Counter {
                            ContentItem::Counter(ident, style)
                        } else {
                            let separator = gecko_function.mSeparator.to_string();
                            ContentItem::Counters(ident, separator.into_boxed_str(), style)
                        }
                    },
                    StyleContentType::Image => {
                        unsafe {
                            let gecko_image_request =
                                &**gecko_content.mContent.mImage.as_ref();
                            ContentItem::Url(
                                ComputedImageUrl::from_image_request(gecko_image_request)
                            )
                        }
                    },
                    _ => panic!("Found unexpected value in style struct for content property"),
                }
            }).collect::<Vec<_>>().into_boxed_slice()
        )
    }

    % for counter_property in ["Increment", "Reset", "Set"]:
        pub fn set_counter_${counter_property.lower()}(
            &mut self,
            v: longhands::counter_${counter_property.lower()}::computed_value::T
        ) {
            unsafe {
                bindings::Gecko_ClearAndResizeCounter${counter_property}s(&mut *self.gecko, v.len() as u32);
                for (i, pair) in v.0.into_vec().into_iter().enumerate() {
                    self.gecko.m${counter_property}s[i].mCounter.set_move(
                        RefPtr::from_addrefed(pair.name.0.into_addrefed())
                    );
                    self.gecko.m${counter_property}s[i].mValue = pair.value;
                }
            }
        }

        pub fn copy_counter_${counter_property.lower()}_from(&mut self, other: &Self) {
            unsafe {
                bindings::Gecko_CopyCounter${counter_property}sFrom(&mut *self.gecko, &*other.gecko)
            }
        }

        pub fn reset_counter_${counter_property.lower()}(&mut self, other: &Self) {
            self.copy_counter_${counter_property.lower()}_from(other)
        }

        pub fn clone_counter_${counter_property.lower()}(
            &self
        ) -> longhands::counter_${counter_property.lower()}::computed_value::T {
            use crate::values::generics::counters::CounterPair;
            use crate::values::CustomIdent;

            longhands::counter_${counter_property.lower()}::computed_value::T::new(
                self.gecko.m${counter_property}s.iter().map(|ref gecko_counter| {
                    CounterPair {
                        name: CustomIdent(unsafe {
                            Atom::from_raw(gecko_counter.mCounter.mRawPtr)
                        }),
                        value: gecko_counter.mValue,
                    }
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
    pub fn clone__moz_box_ordinal_group(&self) -> i32 {
        self.gecko.mBoxOrdinal as i32
    }
</%self:impl_trait>

% for style_struct in data.style_structs:
${declare_style_struct(style_struct)}
${impl_style_struct(style_struct)}
% endfor
