/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// `data` comes from components/style/properties.mako.rs; see build.rs for more details.

<%!
    from data import to_camel_case, to_camel_case_lower
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
use gecko_bindings::bindings::Gecko_CopyCounterStyle;
use gecko_bindings::bindings::Gecko_CopyCursorArrayFrom;
use gecko_bindings::bindings::Gecko_CopyFontFamilyFrom;
use gecko_bindings::bindings::Gecko_CopyImageValueFrom;
use gecko_bindings::bindings::Gecko_CopyListStyleImageFrom;
use gecko_bindings::bindings::Gecko_EnsureImageLayersLength;
use gecko_bindings::bindings::Gecko_SetCursorArrayLength;
use gecko_bindings::bindings::Gecko_SetCursorImageValue;
use gecko_bindings::bindings::Gecko_StyleTransition_SetUnsupportedProperty;
use gecko_bindings::bindings::Gecko_NewCSSShadowArray;
use gecko_bindings::bindings::Gecko_nsStyleFont_SetLang;
use gecko_bindings::bindings::Gecko_nsStyleFont_CopyLangFrom;
use gecko_bindings::bindings::Gecko_SetListStyleImageNone;
use gecko_bindings::bindings::Gecko_SetListStyleImageImageValue;
use gecko_bindings::bindings::Gecko_SetNullImageValue;
use gecko_bindings::bindings::{Gecko_ResetFilters, Gecko_CopyFiltersFrom};
use gecko_bindings::bindings::RawGeckoPresContextBorrowed;
use gecko_bindings::structs;
use gecko_bindings::structs::nsCSSPropertyID;
use gecko_bindings::structs::mozilla::CSSPseudoElementType;
use gecko_bindings::structs::mozilla::CSSPseudoElementType_InheritingAnonBox;
use gecko_bindings::structs::root::NS_STYLE_CONTEXT_TYPE_SHIFT;
use gecko_bindings::sugar::ns_style_coord::{CoordDataValue, CoordData, CoordDataMut};
use gecko::values::convert_nscolor_to_rgba;
use gecko::values::convert_rgba_to_nscolor;
use gecko::values::GeckoStyleCoordConvertible;
use gecko::values::round_border_to_device_pixels;
use logical_geometry::WritingMode;
use media_queries::Device;
use properties::animated_properties::TransitionProperty;
use properties::computed_value_flags::*;
use properties::{longhands, Importance, LonghandId};
use properties::{PropertyDeclaration, PropertyDeclarationBlock, PropertyDeclarationId};
use rule_tree::StrongRuleNode;
use selector_parser::PseudoElement;
use servo_arc::{Arc, RawOffsetArc};
use std::marker::PhantomData;
use std::mem::{forget, uninitialized, transmute, zeroed};
use std::{cmp, ops, ptr};
use values::{self, CustomIdent, Either, KeyframesName, None_};
use values::computed::{NonNegativeLength, ToComputedValue, Percentage};
use values::computed::font::{FontSize, SingleFontFamily};
use values::computed::effects::{BoxShadow, Filter, SimpleShadow};
use values::computed::outline::OutlineStyle;
use values::generics::column::ColumnCount;
use values::generics::position::ZIndex;
use values::generics::text::MozTabSize;
use values::generics::transform::TransformStyle;
use values::generics::url::UrlOrNone;
use computed_values::border_style;

pub mod style_structs {
    % for style_struct in data.style_structs:
    pub use super::${style_struct.gecko_struct_name} as ${style_struct.name};
    % endfor
}

/// FIXME(emilio): This is completely duplicated with the other properties code.
pub type ComputedValuesInner = ::gecko_bindings::structs::ServoComputedData;

#[repr(C)]
pub struct ComputedValues(::gecko_bindings::structs::mozilla::ComputedStyle);

impl ComputedValues {
    pub fn new(
        device: &Device,
        parent: Option<<&ComputedValues>,
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
        ).to_outer(
            device.pres_context(),
            parent,
            pseudo.map(|p| p.pseudo_info())
        )
    }

    pub fn default_values(pres_context: RawGeckoPresContextBorrowed) -> Arc<Self> {
        ComputedValuesInner::new(
            /* custom_properties = */ None,
            /* writing_mode = */ WritingMode::empty(), // FIXME(bz): This seems dubious
            ComputedValueFlags::empty(),
            /* rules = */ None,
            /* visited_style = */ None,
            % for style_struct in data.style_structs:
            style_structs::${style_struct.name}::default(pres_context),
            % endfor
        ).to_outer(pres_context, None, None)
    }

    pub fn pseudo(&self) -> Option<PseudoElement> {
        use string_cache::Atom;

        let atom = (self.0).mPseudoTag.mRawPtr;
        if atom.is_null() {
            return None;
        }

        let atom = unsafe { Atom::from_raw(atom) };
        PseudoElement::from_atom(&atom)
    }

    fn get_pseudo_type(&self) -> CSSPseudoElementType {
        let bits = (self.0).mBits;
        let our_type = bits >> NS_STYLE_CONTEXT_TYPE_SHIFT;
        unsafe { transmute(our_type as u8) }
    }

    pub fn is_anon_box(&self) -> bool {
        let our_type = self.get_pseudo_type();
        return our_type == CSSPseudoElementType_InheritingAnonBox ||
               our_type == CSSPseudoElementType::NonInheritingAnonBox;
    }

    /// Returns true if the display property is changed from 'none' to others.
    pub fn is_display_property_changed_from_none(
        &self,
        old_values: Option<<&ComputedValues>
    ) -> bool {
        use properties::longhands::display::computed_value::T as Display;

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

type PseudoInfo = (*mut structs::nsAtom, structs::CSSPseudoElementType);
type ParentComputedStyleInfo<'a> = Option< &'a ComputedValues>;

impl ComputedValuesInner {
    pub fn new(custom_properties: Option<Arc<CustomPropertiesMap>>,
               writing_mode: WritingMode,
               flags: ComputedValueFlags,
               rules: Option<StrongRuleNode>,
               visited_style: Option<Arc<ComputedValues>>,
               % for style_struct in data.style_structs:
               ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
               % endfor
    ) -> Self {
        ComputedValuesInner {
            custom_properties: custom_properties,
            writing_mode: writing_mode,
            rules: rules,
            visited_style: visited_style.map(|x| Arc::into_raw_offset(x)),
            flags: flags,
            % for style_struct in data.style_structs:
            ${style_struct.gecko_name}: Arc::into_raw_offset(${style_struct.ident}),
            % endfor
        }
    }

    fn to_outer(
        self,
        pres_context: RawGeckoPresContextBorrowed,
        parent: ParentComputedStyleInfo,
        info: Option<PseudoInfo>
    ) -> Arc<ComputedValues> {
        let (tag, ty) = if let Some(info) = info {
            info
        } else {
            (ptr::null_mut(), structs::CSSPseudoElementType::NotPseudo)
        };

        unsafe { self.to_outer_helper(pres_context, parent, ty, tag) }
    }

    unsafe fn to_outer_helper(
        self,
        pres_context: bindings::RawGeckoPresContextBorrowed,
        parent: ParentComputedStyleInfo,
        pseudo_ty: structs::CSSPseudoElementType,
        pseudo_tag: *mut structs::nsAtom
    ) -> Arc<ComputedValues> {
        let arc = {
            let arc: Arc<ComputedValues> = Arc::new(uninitialized());
            bindings::Gecko_ComputedStyle_Init(&arc.0 as *const _ as *mut _,
                                                   parent, pres_context,
                                                   &self, pseudo_ty, pseudo_tag);
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
        !self.get_box().gecko.mBinding.mRawPtr.is_null()
    }

    pub fn to_declaration_block(&self, property: PropertyDeclarationId) -> PropertyDeclarationBlock {
        let value = match property {
            % for prop in data.longhands:
                % if prop.animatable:
                    PropertyDeclarationId::Longhand(LonghandId::${prop.camel_case}) => {
                        PropertyDeclaration::${prop.camel_case}(
                            % if prop.boxed:
                                Box::new(
                            % endif
                            longhands::${prop.ident}::SpecifiedValue::from_computed_value(
                              &self.get_${prop.style_struct.ident.strip("_")}().clone_${prop.ident}())
                            % if prop.boxed:
                                )
                            % endif
                        )
                    },
                % endif
            % endfor
            PropertyDeclarationId::Custom(_name) => unimplemented!(),
            _ => unimplemented!()
        };
        PropertyDeclarationBlock::with_one(value, Importance::Normal)
    }
}

<%def name="declare_style_struct(style_struct)">
pub use ::gecko_bindings::structs::mozilla::Gecko${style_struct.gecko_name} as ${style_struct.gecko_struct_name};
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
        From::from(self.gecko.${gecko_ffi_name})
    }
</%def>

<%def name="impl_simple_copy(ident, gecko_ffi_name, on_set=None, *kwargs)">
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name} = other.gecko.${gecko_ffi_name};
        % if on_set:
        self.${on_set}();
        % endif
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
                Keyword::${to_camel_case(value)} =>
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
    // FIXME: We introduced non_upper_case_globals for -moz-appearance only
    //        since the prefix of Gecko value starts with ThemeWidgetType_NS_THEME.
    //        We should remove this after fix bug 1371809.
    #[allow(non_snake_case, non_upper_case_globals)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use properties::longhands::${ident}::computed_value::T as Keyword;
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

    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }
</%def>

<%def name="impl_color_clone(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        ${get_gecko_property(gecko_ffi_name)}.into()
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

<%def name="impl_position(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        ${set_gecko_property("%s.mXPosition" % gecko_ffi_name, "v.horizontal.into()")}
        ${set_gecko_property("%s.mYPosition" % gecko_ffi_name, "v.vertical.into()")}
    }
    <%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        longhands::${ident}::computed_value::T {
            horizontal: self.gecko.${gecko_ffi_name}.mXPosition.into(),
            vertical: self.gecko.${gecko_ffi_name}.mYPosition.into(),
        }
    }
</%def>

<%def name="impl_color(ident, gecko_ffi_name)">
<%call expr="impl_color_setter(ident, gecko_ffi_name)"></%call>
<%call expr="impl_color_copy(ident, gecko_ffi_name)"></%call>
<%call expr="impl_color_clone(ident, gecko_ffi_name)"></%call>
</%def>

<%def name="impl_rgba_color(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        ${set_gecko_property(gecko_ffi_name, "convert_rgba_to_nscolor(&v)")}
    }
    <%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        convert_nscolor_to_rgba(${get_gecko_property(gecko_ffi_name)})
    }
</%def>

<%def name="impl_svg_length(ident, gecko_ffi_name)">
    // When context-value is used on an SVG length, the corresponding flag is
    // set on mContextFlags, and the length field is set to the initial value.

    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        use values::generics::svg::{SVGLength, SvgLengthOrPercentageOrNumber};
        use gecko_bindings::structs::nsStyleSVG_${ident.upper()}_CONTEXT as CONTEXT_VALUE;
        let length = match v {
            SVGLength::Length(length) => {
                self.gecko.mContextFlags &= !CONTEXT_VALUE;
                length
            }
            SVGLength::ContextValue => {
                self.gecko.mContextFlags |= CONTEXT_VALUE;
                match longhands::${ident}::get_initial_value() {
                    SVGLength::Length(length) => length,
                    _ => unreachable!("Initial value should not be context-value"),
                }
            }
        };
        match length {
            SvgLengthOrPercentageOrNumber::LengthOrPercentage(lop) =>
                self.gecko.${gecko_ffi_name}.set(lop),
            SvgLengthOrPercentageOrNumber::Number(num) =>
                self.gecko.${gecko_ffi_name}.set_value(CoordDataValue::Factor(num.into())),
        }
    }

    pub fn copy_${ident}_from(&mut self, other: &Self) {
        use gecko_bindings::structs::nsStyleSVG_${ident.upper()}_CONTEXT as CONTEXT_VALUE;
        self.gecko.${gecko_ffi_name}.copy_from(&other.gecko.${gecko_ffi_name});
        self.gecko.mContextFlags =
            (self.gecko.mContextFlags & !CONTEXT_VALUE) |
            (other.gecko.mContextFlags & CONTEXT_VALUE);
    }

    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use values::generics::svg::{SVGLength, SvgLengthOrPercentageOrNumber};
        use values::computed::LengthOrPercentage;
        use gecko_bindings::structs::nsStyleSVG_${ident.upper()}_CONTEXT as CONTEXT_VALUE;
        if (self.gecko.mContextFlags & CONTEXT_VALUE) != 0 {
            return SVGLength::ContextValue;
        }
        let length = match self.gecko.${gecko_ffi_name}.as_value() {
            CoordDataValue::Factor(number) =>
                SvgLengthOrPercentageOrNumber::Number(number),
            CoordDataValue::Coord(coord) =>
                SvgLengthOrPercentageOrNumber::LengthOrPercentage(
                    LengthOrPercentage::Length(Au(coord).into())),
            CoordDataValue::Percent(p) =>
                SvgLengthOrPercentageOrNumber::LengthOrPercentage(
                    LengthOrPercentage::Percentage(Percentage(p))),
            CoordDataValue::Calc(calc) =>
                SvgLengthOrPercentageOrNumber::LengthOrPercentage(
                    LengthOrPercentage::Calc(calc.into())),
            _ => unreachable!("Unexpected coordinate in ${ident}"),
        };
        SVGLength::Length(length.into())
    }
</%def>

<%def name="impl_svg_opacity(ident, gecko_ffi_name)">
    <% source_prefix = ident.split("_")[0].upper() + "_OPACITY_SOURCE" %>

    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        use gecko_bindings::structs::nsStyleSVG_${source_prefix}_MASK as MASK;
        use gecko_bindings::structs::nsStyleSVG_${source_prefix}_SHIFT as SHIFT;
        use gecko_bindings::structs::nsStyleSVGOpacitySource::*;
        use values::generics::svg::SVGOpacity;
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
        use gecko_bindings::structs::nsStyleSVG_${source_prefix}_MASK as MASK;
        self.gecko.${gecko_ffi_name} = other.gecko.${gecko_ffi_name};
        self.gecko.mContextFlags =
            (self.gecko.mContextFlags & !MASK) |
            (other.gecko.mContextFlags & MASK);
    }

    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use gecko_bindings::structs::nsStyleSVG_${source_prefix}_MASK as MASK;
        use gecko_bindings::structs::nsStyleSVG_${source_prefix}_SHIFT as SHIFT;
        use gecko_bindings::structs::nsStyleSVGOpacitySource::*;
        use values::generics::svg::SVGOpacity;

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
        use values::generics::svg::SVGPaintKind;
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
                    bindings::Gecko_nsStyleSVGPaint_SetURLValue(paint, url.url_value.get());
                }
            }
            SVGPaintKind::Color(color) => {
                paint.mType = nsStyleSVGPaintType::eStyleSVGPaintType_Color;
                unsafe {
                    *paint.mPaint.mColor.as_mut() = convert_rgba_to_nscolor(&color);
                }
            }
        }

        paint.mFallbackType = match fallback {
            Some(Either::First(color)) => {
                paint.mFallbackColor = convert_rgba_to_nscolor(&color);
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
        use values::generics::svg::{SVGPaint, SVGPaintKind};
        use values::specified::url::SpecifiedUrl;
        use self::structs::nsStyleSVGPaintType;
        use self::structs::nsStyleSVGFallbackType;
        let ref paint = ${get_gecko_property(gecko_ffi_name)};

        let fallback = match paint.mFallbackType {
            nsStyleSVGFallbackType::eStyleSVGFallbackType_Color => {
                Some(Either::First(convert_nscolor_to_rgba(paint.mFallbackColor)))
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
        v.to_gecko_style_coord(&mut self.gecko.${gecko_ffi_name}.data_at_mut(${index}));
    }
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name}.data_at_mut(${index}).copy_from(&other.gecko.${gecko_ffi_name}.data_at(${index}));
    }
    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use properties::longhands::${ident}::computed_value::T;
        T::from_gecko_style_coord(&self.gecko.${gecko_ffi_name}.data_at(${index}))
            .expect("clone for ${ident} failed")
    }
</%def>

<%def name="impl_style_coord(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        v.to_gecko_style_coord(&mut self.gecko.${gecko_ffi_name});
    }
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name}.copy_from(&other.gecko.${gecko_ffi_name});
    }
    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use properties::longhands::${ident}::computed_value::T;
        T::from_gecko_style_coord(&self.gecko.${gecko_ffi_name})
            .expect("clone for ${ident} failed")
    }
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

    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }
</%def>

<%def name="impl_corner_style_coord(ident, gecko_ffi_name, x_index, y_index)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        v.0.width().to_gecko_style_coord(&mut self.gecko.${gecko_ffi_name}.data_at_mut(${x_index}));
        v.0.height().to_gecko_style_coord(&mut self.gecko.${gecko_ffi_name}.data_at_mut(${y_index}));
    }
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name}.data_at_mut(${x_index})
                  .copy_from(&other.gecko.${gecko_ffi_name}.data_at(${x_index}));
        self.gecko.${gecko_ffi_name}.data_at_mut(${y_index})
                  .copy_from(&other.gecko.${gecko_ffi_name}.data_at(${y_index}));
    }
    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

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
</%def>

<%def name="impl_css_url(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        match v {
            UrlOrNone::Url(ref url) => {
                self.gecko.${gecko_ffi_name}.set_move(url.url_value.clone())
            }
            UrlOrNone::None => {
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
    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use values::specified::url::SpecifiedUrl;

        if self.gecko.${gecko_ffi_name}.mRawPtr.is_null() {
            UrlOrNone::none()
        } else {
            unsafe {
                let ref gecko_url_value = *self.gecko.${gecko_ffi_name}.mRawPtr;
                UrlOrNone::Url(SpecifiedUrl::from_url_value_data(&gecko_url_value._base)
                               .expect("${gecko_ffi_name} could not convert to SpecifiedUrl"))
            }
        }
    }
</%def>

<%
transform_functions = [
    ("Matrix3D", "matrix3d", ["number"] * 16),
    ("Matrix", "matrix", ["number"] * 6),
    ("Translate", "translate", ["lop", "optional_lop"]),
    ("Translate3D", "translate3d", ["lop", "lop", "length"]),
    ("TranslateX", "translatex", ["lop"]),
    ("TranslateY", "translatey", ["lop"]),
    ("TranslateZ", "translatez", ["length"]),
    ("Scale3D", "scale3d", ["number"] * 3),
    ("Scale", "scale", ["number", "optional_number"]),
    ("ScaleX", "scalex", ["number"]),
    ("ScaleY", "scaley", ["number"]),
    ("ScaleZ", "scalez", ["number"]),
    ("Rotate", "rotate", ["angle"]),
    ("Rotate3D", "rotate3d", ["number"] * 3 + ["angle"]),
    ("RotateX", "rotatex", ["angle"]),
    ("RotateY", "rotatey", ["angle"]),
    ("RotateZ", "rotatez", ["angle"]),
    ("Skew", "skew", ["angle", "optional_angle"]),
    ("SkewX", "skewx", ["angle"]),
    ("SkewY", "skewy", ["angle"]),
    ("Perspective", "perspective", ["length"]),
    ("InterpolateMatrix", "interpolatematrix", ["list"] * 2 + ["percentage"]),
    ("AccumulateMatrix", "accumulatematrix", ["list"] * 2 + ["integer_to_percentage"])
]
%>

<%def name="transform_function_arm(name, keyword, items)">
    <%
        has_optional = items[-1].startswith("optional_")
        pattern = None
        if keyword == "matrix3d":
            # m11: number1, m12: number2, ..
            single_patterns = ["m%s: %s" % (str(a / 4 + 1) + str(a % 4 + 1), b + str(a + 1)) for (a, b)
                                in enumerate(items)]
            pattern = "(Matrix3D { %s })" % ", ".join(single_patterns)
        elif keyword == "matrix":
            # a: number1, b: number2, ..
            single_patterns = ["%s: %s" % (chr(ord('a') + a), b + str(a + 1)) for (a, b)
                                in enumerate(items)]
            pattern = "(Matrix { %s })" % ", ".join(single_patterns)
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
            "length" : "bindings::Gecko_CSSValue_SetPixelLength(%s, %s.px())",
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
            "list" : "%s.set_shared_list(%s.0.iter().map(&convert_to_ns_css_value));",
        }
    %>
    ::values::generics::transform::TransformOperation::${name}${pattern} => {
        % if has_optional:
            let optional_present = ${items[-1] + str(len(items))}.is_some();
            let len = if optional_present {
                ${len(items) + 1}
            } else {
                ${len(items)}
            };
        % else:
            let len = ${len(items) + 1};
        % endif
        bindings::Gecko_CSSValue_SetFunction(gecko_value, len);
        bindings::Gecko_CSSValue_SetKeyword(
            bindings::Gecko_CSSValue_GetArrayItem(gecko_value, 0),
            structs::nsCSSKeyword::eCSSKeyword_${keyword}
        );
        % for index, item in enumerate(items):
            <% replaced_item = item.replace("optional_", "") %>
            % if item.startswith("optional"):
                if let Some(${replaced_item + str(index + 1)}) = ${item + str(index + 1)} {
            % endif
            % if item == "list":
                debug_assert!(!${item}${index + 1}.0.is_empty());
            % endif
            ${css_value_setters[replaced_item] % (
                "bindings::Gecko_CSSValue_GetArrayItem(gecko_value, %d)" % (index + 1),
                replaced_item + str(index + 1)
            )};
            % if item.startswith("optional"):
                }
            % endif
        % endfor
    }
</%def>

<%def name="computed_operation_arm(name, keyword, items)">
    <%
        # %s is substituted with the call to GetArrayItem.
        css_value_getters = {
            "length" : "Length::new(bindings::Gecko_CSSValue_GetNumber(%s))",
            "lop" : "%s.get_lop()",
            "lopon" : "Either::Second(%s.get_lop())",
            "lon" : "Either::First(%s.get_length())",
            "angle" : "%s.get_angle()",
            "number" : "bindings::Gecko_CSSValue_GetNumber(%s)",
            "percentage" : "Percentage(bindings::Gecko_CSSValue_GetPercentage(%s))",
            "integer_to_percentage" : "bindings::Gecko_CSSValue_GetPercentage(%s) as i32",
            "list" : "Transform(convert_shared_list_to_operations(%s))",
        }
        pre_symbols = "("
        post_symbols = ")"
        if keyword == "interpolatematrix" or keyword == "accumulatematrix":
            # We generate this like: "TransformOperation::InterpolateMatrix {", so the space is
            # between "InterpolateMatrix"/"AccumulateMatrix" and '{'
            pre_symbols = " {"
            post_symbols = "}"
        elif keyword == "matrix3d":
            pre_symbols = "(Matrix3D {"
            post_symbols = "})"
        elif keyword == "matrix":
            pre_symbols = "(Matrix {"
            post_symbols = "})"
        field_names = None
        if keyword == "interpolatematrix":
            field_names = ["from_list", "to_list", "progress"]
        elif keyword == "accumulatematrix":
            field_names = ["from_list", "to_list", "count"]

    %>
    structs::nsCSSKeyword::eCSSKeyword_${keyword} => {
        ::values::generics::transform::TransformOperation::${name}${pre_symbols}
        % for index, item in enumerate(items):
            % if keyword == "matrix3d":
                m${index / 4 + 1}${index % 4 + 1}:
            % elif keyword == "matrix":
                ${chr(ord('a') + index)}:
            % elif keyword == "interpolatematrix" or keyword == "accumulatematrix":
                ${field_names[index]}:
            % endif
            <%
                getter = css_value_getters[item.replace("optional_", "")] % (
                    "bindings::Gecko_CSSValue_GetArrayItemConst(gecko_value, %d)" % (index + 1)
                )
            %>
            % if item.startswith("optional_"):
                if (**gecko_value.mValue.mArray.as_ref()).mCount == ${index + 1} {
                    None
                } else {
                    Some(${getter})
                }
            % else:
                ${getter}
            % endif
,
        % endfor
        ${post_symbols}
    },
</%def>

fn set_single_transform_function(
    servo_value: &values::computed::TransformOperation,
    gecko_value: &mut structs::nsCSSValue /* output */
) {
    use values::computed::TransformOperation;
    use values::generics::transform::{Matrix, Matrix3D};

    let convert_to_ns_css_value = |item: &TransformOperation| -> structs::nsCSSValue {
        let mut value = structs::nsCSSValue::null();
        set_single_transform_function(item, &mut value);
        value
    };

    unsafe {
        match *servo_value {
            % for servo, gecko, format in transform_functions:
                ${transform_function_arm(servo, gecko, format)}
            % endfor
        }
    }
}

pub fn convert_transform(
    input: &[values::computed::TransformOperation],
    output: &mut structs::root::RefPtr<structs::root::nsCSSValueSharedList>
) {
    use gecko_bindings::sugar::refptr::RefPtr;

    unsafe { output.clear() };

    let list = unsafe {
        RefPtr::from_addrefed(bindings::Gecko_NewCSSValueSharedList(input.len() as u32))
    };
    let value_list = unsafe { list.mHead.as_mut() };
    if let Some(value_list) = value_list {
        for (gecko, servo) in value_list.into_iter().zip(input.into_iter()) {
            set_single_transform_function(servo, gecko);
        }
    }
    output.set_move(list);
}

fn clone_single_transform_function(
    gecko_value: &structs::nsCSSValue
) -> values::computed::TransformOperation {
    use values::computed::{Length, Percentage, TransformOperation};
    use values::generics::transform::{Matrix, Matrix3D};
    use values::generics::transform::Transform;

    let convert_shared_list_to_operations = |value: &structs::nsCSSValue|
                                            -> Vec<TransformOperation> {
        debug_assert_eq!(value.mUnit, structs::nsCSSUnit::eCSSUnit_SharedList);
        let value_list = unsafe {
            value.mValue.mSharedList.as_ref()
                    .as_mut().expect("List pointer should be non-null").mHead.as_ref()
        };
        debug_assert!(value_list.is_some(), "An empty shared list is not allowed");
        value_list.unwrap().into_iter()
                            .map(|item| clone_single_transform_function(item))
                            .collect()
    };

    let transform_function = unsafe {
        bindings::Gecko_CSSValue_GetKeyword(bindings::Gecko_CSSValue_GetArrayItemConst(gecko_value, 0))
    };

    unsafe {
        match transform_function {
            % for servo, gecko, format in transform_functions:
                ${computed_operation_arm(servo, gecko, format)}
            % endfor
            _ => panic!("unacceptable transform function"),
        }
    }
}

pub fn clone_transform_from_list(
    list: Option< &structs::root::nsCSSValueList>
) -> values::computed::Transform {
    use values::generics::transform::Transform;

    let result = match list {
        Some(list) => {
            list.into_iter()
                .filter_map(|value| {
                    // Handle none transform.
                    if value.is_none() {
                        None
                    } else {
                        Some(clone_single_transform_function(value))
                    }
                })
                .collect::<Vec<_>>()
        },
        _ => vec![],
    };
    Transform(result)
}

<%def name="impl_transform(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, other: values::computed::Transform) {
        use gecko_properties::convert_transform;
        if other.0.is_empty() {
            unsafe {
                self.gecko.${gecko_ffi_name}.clear();
            }
            return;
        };
        convert_transform(&other.0, &mut self.gecko.${gecko_ffi_name});
    }

    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        unsafe { self.gecko.${gecko_ffi_name}.set(&other.gecko.${gecko_ffi_name}); }
    }

    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> values::computed::Transform {
        use gecko_properties::clone_transform_from_list;
        use values::generics::transform::Transform;

        if self.gecko.${gecko_ffi_name}.mRawPtr.is_null() {
            return Transform(vec!());
        }
        let list = unsafe { (*self.gecko.${gecko_ffi_name}.to_safe().get()).mHead.as_ref() };
        clone_transform_from_list(list)
    }
</%def>

<%def name="impl_transform_origin(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: values::computed::TransformOrigin) {
        self.gecko.${gecko_ffi_name}[0].set(v.horizontal);
        self.gecko.${gecko_ffi_name}[1].set(v.vertical);
        // transform-origin supports the third value for depth, while
        // -moz-window-transform-origin doesn't. The following code is
        // for handling this difference. If we can have more knowledge
        // about the type here, we may want to check that the length is
        // exactly either 2 or 3 in compile time.
        if let Some(third) = self.gecko.${gecko_ffi_name}.get_mut(2) {
            third.set(v.depth);
        }
    }

    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name}[0].copy_from(&other.gecko.${gecko_ffi_name}[0]);
        self.gecko.${gecko_ffi_name}[1].copy_from(&other.gecko.${gecko_ffi_name}[1]);
        if let (Some(self_third), Some(other_third)) =
            (self.gecko.${gecko_ffi_name}.get_mut(2), other.gecko.${gecko_ffi_name}.get(2))
        {
            self_third.copy_from(other_third)
        }
    }

    #[allow(non_snake_case)]
    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> values::computed::TransformOrigin {
        use values::computed::{Length, LengthOrPercentage, TransformOrigin};
        TransformOrigin {
            horizontal: LengthOrPercentage::from_gecko_style_coord(&self.gecko.${gecko_ffi_name}[0])
                .expect("clone for LengthOrPercentage failed"),
            vertical: LengthOrPercentage::from_gecko_style_coord(&self.gecko.${gecko_ffi_name}[1])
                .expect("clone for LengthOrPercentage failed"),
            depth: if let Some(third) = self.gecko.${gecko_ffi_name}.get(2) {
                Length::from_gecko_style_coord(third)
                    .expect("clone for Length failed")
            } else {
                Length::new(0.)
            },
        }
    }
</%def>

<%def name="impl_logical(name, **kwargs)">
    ${helpers.logical_setter(name)}
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

<%def name="impl_font_settings(ident, tag_type, value_type, gecko_value_type)">
    <%
    gecko_ffi_name = to_camel_case_lower(ident)
    %>

    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        let current_settings = &mut self.gecko.mFont.${gecko_ffi_name};
        current_settings.clear_pod();

        unsafe { current_settings.set_len_pod(v.0.len() as u32) };

        for (current, other) in current_settings.iter_mut().zip(v.0.iter()) {
            current.mTag = other.tag.0;
            current.mValue = other.value as ${gecko_value_type};
        }
    }

    pub fn copy_${ident}_from(&mut self, other: &Self) {
        let current_settings = &mut self.gecko.mFont.${gecko_ffi_name};
        let other_settings = &other.gecko.mFont.${gecko_ffi_name};
        let settings_length = other_settings.len() as u32;

        current_settings.clear_pod();
        unsafe { current_settings.set_len_pod(settings_length) };

        for (current, other) in current_settings.iter_mut().zip(other_settings.iter()) {
            current.mTag = other.mTag;
            current.mValue = other.mValue;
        }
    }

    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use values::generics::font::{FontSettings, FontTag, ${tag_type}};

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
        "Color": impl_color,
        "GreaterThanOrEqualToOneNumber": impl_simple,
        "Integer": impl_simple,
        "length::LengthOrAuto": impl_style_coord,
        "length::LengthOrNormal": impl_style_coord,
        "length::NonNegativeLengthOrAuto": impl_style_coord,
        "length::NonNegativeLengthOrPercentageOrNormal": impl_style_coord,
        "FlexBasis": impl_style_coord,
        "Length": impl_absolute_length,
        "LengthOrNormal": impl_style_coord,
        "LengthOrPercentage": impl_style_coord,
        "LengthOrPercentageOrAuto": impl_style_coord,
        "LengthOrPercentageOrNone": impl_style_coord,
        "MaxLength": impl_style_coord,
        "MozLength": impl_style_coord,
        "MozScriptMinSize": impl_absolute_length,
        "MozScriptSizeMultiplier": impl_simple,
        "NonNegativeLengthOrPercentage": impl_style_coord,
        "NonNegativeNumber": impl_simple,
        "Number": impl_simple,
        "Opacity": impl_simple,
        "Perspective": impl_style_coord,
        "Position": impl_position,
        "RGBAColor": impl_rgba_color,
        "SVGLength": impl_svg_length,
        "SVGOpacity": impl_svg_opacity,
        "SVGPaint": impl_svg_paint,
        "SVGWidth": impl_svg_length,
        "Transform": impl_transform,
        "TransformOrigin": impl_transform_origin,
        "url::UrlOrNone": impl_css_url,
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
        else:
            method = predefined_types[longhand.predefined_type]

        method(**args)

    picked_longhands = []
    for x in longhands:
        if x.keyword or x.predefined_type in predefined_types or x.logical:
            picked_longhands.append(x)
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
}
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

<%self:impl_trait style_struct_name="Border"
                  skip_longhands="${skip_border_longhands} border-image-source border-image-outset
                                  border-image-repeat border-image-width border-image-slice">
    % for side in SIDES:
    <% impl_keyword("border_%s_style" % side.ident,
                    "mBorderStyle[%s]" % side.index,
                    border_style_keyword,
                    on_set="update_border_%s" % side.ident) %>

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

    <% impl_color("border_%s_color" % side.ident, "(mBorderColor)[%s]" % side.index) %>

    <% impl_non_negative_length("border_%s_width" % side.ident,
                                "mComputedBorder.%s" % side.ident,
                                inherit_from="mBorder.%s" % side.ident,
                                round_to_pixels=True) %>

    pub fn border_${side.ident}_has_nonzero_width(&self) -> bool {
        self.gecko.mComputedBorder.${side.ident} != 0
    }
    % endfor

    % for corner in CORNERS:
    <% impl_corner_style_coord("border_%s_radius" % corner.ident,
                               "mBorderRadius",
                               corner.x_index,
                               corner.y_index) %>
    % endfor

    pub fn set_border_image_source(&mut self, image: longhands::border_image_source::computed_value::T) {
        unsafe {
            // Prevent leaking of the last elements we did set
            Gecko_SetNullImageValue(&mut self.gecko.mBorderImageSource);
        }

        if let Either::Second(image) = image {
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
        use values::specified::border::BorderImageRepeatKeyword;
        use gecko_bindings::structs::StyleBorderImageRepeat;

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
        use values::specified::border::BorderImageRepeatKeyword;
        use gecko_bindings::structs::StyleBorderImageRepeat;

        % for side in ["H", "V"]:
        let servo_${side.lower()} = match self.gecko.mBorderImageRepeat${side} {
            % for keyword in border_image_repeat_keywords:
            StyleBorderImageRepeat::${keyword} => BorderImageRepeatKeyword::${keyword},
            % endfor
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
                    .expect("mBorderImageSlice[${side}] could not convert to NumberOrPercentageRect"),
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
                              side.index) %>
    % endfor
</%self:impl_trait>

<% skip_padding_longhands = " ".join(["padding-%s" % x.ident for x in SIDES]) %>
<%self:impl_trait style_struct_name="Padding"
                  skip_longhands="${skip_padding_longhands}">

    % for side in SIDES:
    <% impl_split_style_coord("padding_%s" % side.ident,
                              "mPadding",
                              side.index) %>
    % endfor
</%self:impl_trait>

<% skip_position_longhands = " ".join(x.ident for x in SIDES + GRID_LINES) %>
<%self:impl_trait style_struct_name="Position"
                  skip_longhands="${skip_position_longhands} z-index order
                                  align-content justify-content align-self
                                  justify-self align-items justify-items
                                  grid-auto-rows grid-auto-columns
                                  grid-auto-flow grid-template-areas
                                  grid-template-rows grid-template-columns">
    % for side in SIDES:
    <% impl_split_style_coord(side.ident, "mOffset", side.index) %>
    % endfor

    pub fn set_z_index(&mut self, v: longhands::z_index::computed_value::T) {
        match v {
            ZIndex::Integer(n) => self.gecko.mZIndex.set_value(CoordDataValue::Integer(n)),
            ZIndex::Auto => self.gecko.mZIndex.set_value(CoordDataValue::Auto),
        }
    }

    pub fn copy_z_index_from(&mut self, other: &Self) {
        use gecko_bindings::structs::nsStyleUnit;
        // z-index is never a calc(). If it were, we'd be leaking here, so
        // assert that it isn't.
        debug_assert_ne!(self.gecko.mZIndex.unit(), nsStyleUnit::eStyleUnit_Calc);
        unsafe {
            self.gecko.mZIndex.copy_from_unchecked(&other.gecko.mZIndex);
        }
    }

    pub fn reset_z_index(&mut self, other: &Self) {
        self.copy_z_index_from(other)
    }

    pub fn clone_z_index(&self) -> longhands::z_index::computed_value::T {
        return match self.gecko.mZIndex.as_value() {
            CoordDataValue::Integer(n) => ZIndex::Integer(n),
            CoordDataValue::Auto => ZIndex::Auto,
            _ => {
                debug_assert!(false);
                ZIndex::Integer(0)
            }
        }
    }

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
        debug_assert_ne!(v.0, ::values::specified::align::AlignFlags::LEGACY);
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
        use gecko_bindings::structs::{nsStyleGridLine_kMinLine, nsStyleGridLine_kMaxLine};

        let ident = v.ident.as_ref().map_or(&[] as &[_], |ident| ident.0.as_slice());
        self.gecko.${value.gecko}.mLineName.assign(ident);
        self.gecko.${value.gecko}.mHasSpan = v.is_span;
        if let Some(integer) = v.line_num {
            // clamping the integer between a range
            self.gecko.${value.gecko}.mInteger = cmp::max(nsStyleGridLine_kMinLine,
                cmp::min(integer, nsStyleGridLine_kMaxLine));
        }
    }

    pub fn copy_${value.name}_from(&mut self, other: &Self) {
        self.gecko.${value.gecko}.mHasSpan = other.gecko.${value.gecko}.mHasSpan;
        self.gecko.${value.gecko}.mInteger = other.gecko.${value.gecko}.mInteger;
        self.gecko.${value.gecko}.mLineName.assign(&*other.gecko.${value.gecko}.mLineName);
    }

    pub fn reset_${value.name}(&mut self, other: &Self) {
        self.copy_${value.name}_from(other)
    }

    pub fn clone_${value.name}(&self) -> longhands::${value.name}::computed_value::T {
        use gecko_bindings::structs::{nsStyleGridLine_kMinLine, nsStyleGridLine_kMaxLine};
        use string_cache::Atom;

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
                    Some(self.gecko.${value.gecko}.mInteger)
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

    pub fn reset_grid_auto_${kind}(&mut self, other: &Self) {
        self.copy_grid_auto_${kind}_from(other)
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
                        bindings::Gecko_ResizeTArrayForStrings(
                            &mut value.mRepeatAutoLineNameListBefore, 0);
                        bindings::Gecko_ResizeTArrayForStrings(
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
        use Atom;
        use gecko_bindings::structs::nsTArray;
        use nsstring::nsStringRepr;
        use values::CustomIdent;
        use values::generics::grid::{GridTemplateComponent, LineNameList, RepeatCount};
        use values::generics::grid::{TrackList, TrackListType, TrackListValue, TrackRepeat, TrackSize};

        let value = match unsafe { ${self_grid}.mPtr.as_ref() } {
            None => return GridTemplateComponent::None,
            Some(value) => value,
        };

        #[inline]
        fn to_boxed_customident_slice(gecko_names: &nsTArray<nsStringRepr>) -> Box<[CustomIdent]> {
            let idents: Vec<CustomIdent> = gecko_names.iter().map(|gecko_name| {
                CustomIdent(Atom::from(gecko_name.to_string()))
            }).collect();
            idents.into_boxed_slice()
        }

        #[inline]
        fn to_line_names_vec(gecko_line_names: &nsTArray<nsTArray<nsStringRepr>>)
            -> Vec<Box<[CustomIdent]>> {
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

    pub fn set_grid_template_areas(&mut self, v: values::computed::position::GridTemplateAreas) {
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
                Gecko_NewGridTemplateAreasValue(v.0.areas.len() as u32, v.0.strings.len() as u32, v.0.width))
        };

        for (servo, gecko) in v.0.areas.into_iter().zip(refptr.mNamedAreas.iter_mut()) {
            gecko.mName.assign_utf8(&*servo.name);
            gecko.mColumnStart = servo.columns.start;
            gecko.mColumnEnd = servo.columns.end;
            gecko.mRowStart = servo.rows.start;
            gecko.mRowEnd = servo.rows.end;
        }

        for (servo, gecko) in v.0.strings.into_iter().zip(refptr.mTemplates.iter_mut()) {
            gecko.assign_utf8(&*servo);
        }

        self.gecko.mGridTemplateAreas.set_move(refptr.get())
    }

    pub fn copy_grid_template_areas_from(&mut self, other: &Self) {
        unsafe { self.gecko.mGridTemplateAreas.set(&other.gecko.mGridTemplateAreas) }
    }

    pub fn reset_grid_template_areas(&mut self, other: &Self) {
        self.copy_grid_template_areas_from(other)
    }

    pub fn clone_grid_template_areas(&self) -> values::computed::position::GridTemplateAreas {
        use std::ops::Range;
        use values::None_;
        use values::specified::position::{NamedArea, TemplateAreas, TemplateAreasArc};

        if self.gecko.mGridTemplateAreas.mRawPtr.is_null() {
            return Either::Second(None_);
        }

        let gecko_grid_template_areas = self.gecko.mGridTemplateAreas.mRawPtr;
        let areas = unsafe {
            let vec: Vec<NamedArea> =
                (*gecko_grid_template_areas).mNamedAreas.iter().map(|gecko_name_area| {
                    let name = gecko_name_area.mName.to_string().into_boxed_str();
                    let rows = Range {
                        start: gecko_name_area.mRowStart,
                        end: gecko_name_area.mRowEnd
                    };
                    let columns = Range {
                        start: gecko_name_area.mColumnStart,
                        end: gecko_name_area.mColumnEnd
                    };
                    NamedArea{ name, rows, columns }
                }).collect();
            vec.into_boxed_slice()
        };

        let strings = unsafe {
            let vec: Vec<Box<str>> =
                (*gecko_grid_template_areas).mTemplates.iter().map(|gecko_template| {
                    gecko_template.to_string().into_boxed_str()
                }).collect();
            vec.into_boxed_slice()
        };

        let width = unsafe {
            (*gecko_grid_template_areas).mNColumns
        };

        Either::First(TemplateAreasArc(Arc::new(TemplateAreas{ areas, strings, width })))
    }

</%self:impl_trait>

<% skip_outline_longhands = " ".join("outline-style outline-width".split() +
                                     ["-moz-outline-radius-{0}".format(x.ident.replace("_", ""))
                                      for x in CORNERS]) %>
<%self:impl_trait style_struct_name="Outline"
                  skip_longhands="${skip_outline_longhands}">

    #[allow(non_snake_case)]
    pub fn set_outline_style(&mut self, v: longhands::outline_style::computed_value::T) {
        // FIXME(bholley): Align binary representations and ditch |match| for
        // cast + static_asserts
        let result = match v {
            % for value in border_style_keyword.values_for('gecko'):
                OutlineStyle::Other(border_style::T::${to_camel_case(value)}) =>
                    structs::${border_style_keyword.gecko_constant(value)} ${border_style_keyword.maybe_cast("u8")},
            % endfor
                OutlineStyle::Auto =>
                    structs::${border_style_keyword.gecko_constant('auto')} ${border_style_keyword.maybe_cast("u8")},
        };
        ${set_gecko_property("mOutlineStyle", "result")}

        // NB: This is needed to correctly handling the initial value of
        // outline-width when outline-style changes, see the
        // update_border_${side.ident} comment for more details.
        self.gecko.mActualOutlineWidth = self.gecko.mOutlineWidth;
    }

    #[allow(non_snake_case)]
    pub fn copy_outline_style_from(&mut self, other: &Self) {
        self.gecko.mOutlineStyle = other.gecko.mOutlineStyle;
    }

    #[allow(non_snake_case)]
    pub fn reset_outline_style(&mut self, other: &Self) {
        self.copy_outline_style_from(other)
    }

    #[allow(non_snake_case)]
    pub fn clone_outline_style(&self) -> longhands::outline_style::computed_value::T {
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        match ${get_gecko_property("mOutlineStyle")} ${border_style_keyword.maybe_cast("u32")} {
            % for value in border_style_keyword.values_for('gecko'):
            structs::${border_style_keyword.gecko_constant(value)} => {
                OutlineStyle::Other(border_style::T::${to_camel_case(value)})
            },
            % endfor
            structs::${border_style_keyword.gecko_constant('auto')} => OutlineStyle::Auto,
            % if border_style_keyword.gecko_inexhaustive:
            _ => panic!("Found unexpected value in style struct for outline_style property"),
            % endif
        }
    }

    <% impl_non_negative_length("outline_width", "mActualOutlineWidth",
                                inherit_from="mOutlineWidth",
                                round_to_pixels=True) %>

    % for corner in CORNERS:
    <% impl_corner_style_coord("_moz_outline_radius_%s" % corner.ident.replace("_", ""),
                               "mOutlineRadius",
                               corner.x_index,
                               corner.y_index) %>
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
                             -moz-min-font-size-ratio -x-text-zoom"""
%>
<%self:impl_trait style_struct_name="Font"
    skip_longhands="${skip_font_longhands}">

    // Negative numbers are invalid at parse time, but <integer> is still an
    // i32.
    <% impl_font_settings("font_feature_settings", "FeatureTagValue", "i32", "u32") %>
    <% impl_font_settings("font_variation_settings", "VariationValue", "f32", "f32") %>

    pub fn fixup_none_generic(&mut self, device: &Device) {
        self.gecko.mFont.systemFont = false;
        unsafe {
            bindings::Gecko_nsStyleFont_FixupNoneGeneric(&mut self.gecko, device.pres_context())
        }
    }

    pub fn fixup_system(&mut self, default_font_type: structs::FontFamilyType) {
        self.gecko.mFont.systemFont = true;
        self.gecko.mGenericID = structs::kGenericFont_NONE;
        self.gecko.mFont.fontlist.mDefaultFontType = default_font_type;
    }

    pub fn set_font_family(&mut self, v: longhands::font_family::computed_value::T) {
        self.gecko.mGenericID = structs::kGenericFont_NONE;
        if let Some(generic) = v.0.single_generic() {
            self.gecko.mGenericID = generic;
        }
        self.gecko.mFont.fontlist.mFontlist.mBasePtr.set_move((v.0).0.clone());
    }

    pub fn font_family_count(&self) -> usize {
        0
    }

    pub fn font_family_at(&self, _: usize) -> SingleFontFamily {
        unimplemented!()
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
        use gecko_bindings::structs::FontFamilyType;
        use values::computed::font::{FontFamily, SingleFontFamily, FontFamilyList};

        let fontlist = &self.gecko.mFont.fontlist;
        let shared_fontlist = unsafe { fontlist.mFontlist.mBasePtr.to_safe() };

        if shared_fontlist.mNames.is_empty() {
            let default = match fontlist.mDefaultFontType {
                FontFamilyType::eFamily_serif => {
                    SingleFontFamily::Generic(atom!("serif"))
                }
                FontFamilyType::eFamily_sans_serif => {
                    SingleFontFamily::Generic(atom!("sans-serif"))
                }
                _ => panic!("Default generic must be serif or sans-serif"),
            };
            FontFamily(FontFamilyList::new(Box::new([default])))
        } else {
            FontFamily(FontFamilyList(shared_fontlist))
        }
    }

    pub fn unzoom_fonts(&mut self, device: &Device) {
        self.gecko.mSize = device.unzoom_text(Au(self.gecko.mSize)).0;
        self.gecko.mScriptUnconstrainedSize = device.unzoom_text(Au(self.gecko.mScriptUnconstrainedSize)).0;
        self.gecko.mFont.size = device.unzoom_text(Au(self.gecko.mFont.size)).0;
    }

    pub fn set_font_size(&mut self, v: FontSize) {
        use values::generics::font::KeywordSize;
        self.gecko.mSize = v.size().0;
        self.gecko.mScriptUnconstrainedSize = v.size().0;
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

    /// Set font size, taking into account scriptminsize and scriptlevel
    /// Returns Some(size) if we have to recompute the script unconstrained size
    pub fn apply_font_size(
        &mut self,
        v: FontSize,
        parent: &Self,
        device: &Device,
    ) -> Option<NonNegativeLength> {
        let (adjusted_size, adjusted_unconstrained_size) =
            self.calculate_script_level_size(parent, device);
        // In this case, we have been unaffected by scriptminsize, ignore it
        if parent.gecko.mSize == parent.gecko.mScriptUnconstrainedSize &&
           adjusted_size == adjusted_unconstrained_size {
            self.set_font_size(v);
            self.fixup_font_min_size(device);
            None
        } else {
            self.gecko.mSize = v.size().0;
            self.fixup_font_min_size(device);
            Some(Au(parent.gecko.mScriptUnconstrainedSize).into())
        }
    }

    pub fn fixup_font_min_size(&mut self, device: &Device) {
        unsafe { bindings::Gecko_nsStyleFont_FixupMinFontSize(&mut self.gecko, device.pres_context()) }
    }

    pub fn apply_unconstrained_font_size(&mut self, v: NonNegativeLength) {
        self.gecko.mScriptUnconstrainedSize = v.0.to_i32_au();
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
    pub fn calculate_script_level_size(&self, parent: &Self, device: &Device) -> (Au, Au) {
        use std::cmp;

        let delta = self.gecko.mScriptLevel.saturating_sub(parent.gecko.mScriptLevel);

        let parent_size = Au(parent.gecko.mSize);
        let parent_unconstrained_size = Au(parent.gecko.mScriptUnconstrainedSize);

        if delta == 0 {
            return (parent_size, parent_unconstrained_size)
        }


        let mut min = Au(parent.gecko.mScriptMinSize);
        if self.gecko.mAllowZoom {
            min = device.zoom_text(min);
        }

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
    pub fn inherit_font_size_from(&mut self, parent: &Self,
                                  kw_inherited_size: Option<NonNegativeLength>,
                                  device: &Device) {
        let (adjusted_size, adjusted_unconstrained_size)
            = self.calculate_script_level_size(parent, device);
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

            // Technically the MathML constrained size may also be keyword-derived
            // but we ignore this since it would be too complicated
            // to correctly track and it's mostly unnecessary.
            self.gecko.mFontSizeKeyword = structs::NS_STYLE_FONT_SIZE_NO_KEYWORD as u8;
            self.gecko.mFontSizeFactor = 1.;
            self.gecko.mFontSizeOffset = 0;

            self.gecko.mScriptUnconstrainedSize = adjusted_unconstrained_size.0;
        } else if let Some(size) = kw_inherited_size {
            // Parent element was a keyword-derived size.
            self.gecko.mSize = size.0.to_i32_au();
            // Copy keyword info over.
            self.gecko.mFontSizeFactor = parent.gecko.mFontSizeFactor;
            self.gecko.mFontSizeOffset = parent.gecko.mFontSizeOffset;
            self.gecko.mFontSizeKeyword = parent.gecko.mFontSizeKeyword;
            // MathML constraints didn't apply here, so we can ignore this.
            self.gecko.mScriptUnconstrainedSize = size.0.to_i32_au();
        } else {
            // MathML isn't affecting us, and our parent element does not
            // have a keyword-derived size. Set things normally.
            self.gecko.mSize = parent.gecko.mSize;
            // copy keyword info over
            self.gecko.mFontSizeKeyword = structs::NS_STYLE_FONT_SIZE_NO_KEYWORD as u8;
            self.gecko.mFontSizeFactor = 1.;
            self.gecko.mFontSizeOffset = 0;
            self.gecko.mScriptUnconstrainedSize = parent.gecko.mScriptUnconstrainedSize;
        }
        self.fixup_font_min_size(device);
    }

    pub fn clone_font_size(&self) -> FontSize {
        use values::generics::font::{KeywordInfo, KeywordSize};
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
                    size: size,
                    keyword_info: None,
                }
            }
            _ => unreachable!("mFontSizeKeyword should be an absolute keyword or NO_KEYWORD")
        };
        FontSize {
            size: size,
            keyword_info: Some(KeywordInfo {
                kw: kw,
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
        unsafe { bindings::Gecko_FontStretch_SetFloat(&mut self.gecko.mFont.stretch, (v.0).0) };
    }
    ${impl_simple_copy('font_stretch', 'mFont.stretch')}
    pub fn clone_font_stretch(&self) -> longhands::font_stretch::computed_value::T {
        use values::computed::Percentage;
        use values::generics::NonNegative;

        let stretch =
            unsafe { bindings::Gecko_FontStretch_ToFloat(self.gecko.mFont.stretch) };
        debug_assert!(stretch >= 0.);

        NonNegative(Percentage(stretch))
    }

    pub fn set_font_style(&mut self, v: longhands::font_style::computed_value::T) {
        use values::generics::font::FontStyle;
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
        use values::computed::font::FontStyle;
        FontStyle::from_gecko(self.gecko.mFont.style)
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

    pub fn reset_font_size_adjust(&mut self, other: &Self) {
        self.copy_font_size_adjust_from(other)
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
    pub fn reset__x_lang(&mut self, other: &Self) {
        self.copy__x_lang_from(other)
    }

    ${impl_simple("_moz_script_level", "mScriptLevel")}
    <% impl_simple_type_with_conversion("font_language_override", "mFont.languageOverride") %>

    pub fn set_font_variant_alternates(&mut self,
                                       v: values::computed::font::FontVariantAlternates,
                                       device: &Device) {
        use gecko_bindings::bindings::{Gecko_ClearAlternateValues, Gecko_AppendAlternateValues};
        use gecko_bindings::bindings::Gecko_nsFont_ResetFontFeatureValuesLookup;
        use gecko_bindings::bindings::Gecko_nsFont_SetFontFeatureValuesLookup;
        % for value in "normal swash stylistic ornaments annotation styleset character_variant historical".split():
            use gecko_bindings::structs::NS_FONT_VARIANT_ALTERNATES_${value.upper()};
        % endfor
        use values::specified::font::VariantAlternates;

        unsafe {
            Gecko_ClearAlternateValues(&mut self.gecko.mFont, v.len());
        }

        if v.0.is_empty() {
            self.gecko.mFont.variantAlternates = NS_FONT_VARIANT_ALTERNATES_NORMAL as u16;
            unsafe { Gecko_nsFont_ResetFontFeatureValuesLookup(&mut self.gecko.mFont); }
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

        unsafe {
            Gecko_nsFont_SetFontFeatureValuesLookup(&mut self.gecko.mFont, device.pres_context());
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

    pub fn reset_font_variant_alternates(&mut self, other: &Self) {
        self.copy_font_variant_alternates_from(other)
    }

    pub fn clone_font_variant_alternates(&self) -> values::computed::font::FontVariantAlternates {
        use Atom;
        % for value in "normal swash stylistic ornaments annotation styleset character_variant historical".split():
            use gecko_bindings::structs::NS_FONT_VARIANT_ALTERNATES_${value.upper()};
        % endfor
        use values::specified::font::VariantAlternates;
        use values::specified::font::VariantAlternatesList;
        use values::CustomIdent;

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

<%def name="impl_copy_animation_or_transition_value(type, ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn copy_${type}_${ident}_from(&mut self, other: &Self) {
        self.gecko.m${type.capitalize()}s.ensure_len(other.gecko.m${type.capitalize()}s.len());

        let count = other.gecko.m${type.capitalize()}${gecko_ffi_name}Count;
        self.gecko.m${type.capitalize()}${gecko_ffi_name}Count = count;

        let iter = self.gecko.m${type.capitalize()}s.iter_mut().take(count as usize).zip(
            other.gecko.m${type.capitalize()}s.iter()
        );

        for (ours, others) in iter {
            ours.m${gecko_ffi_name} = others.m${gecko_ffi_name};
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
        debug_assert_ne!(v.len(), 0);
        let input_len = v.len();
        self.gecko.m${type.capitalize()}s.ensure_len(input_len);

        self.gecko.m${type.capitalize()}TimingFunctionCount = input_len as u32;
        for (gecko, servo) in self.gecko.m${type.capitalize()}s.iter_mut().take(input_len as usize).zip(v) {
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
        use properties::longhands::animation_${ident}::single_value::computed_value::T as Keyword;
        match self.gecko.mAnimations[index].m${gecko_ffi_name} ${keyword.maybe_cast("u32")} {
            % for value in keyword.gecko_values():
                structs::${keyword.gecko_constant(value)} => Keyword::${to_camel_case(value)},
            % endfor
            _ => panic!("Found unexpected value for animation-${ident}"),
        }
    }
    ${impl_animation_count(ident, gecko_ffi_name)}
    ${impl_copy_animation_value(ident, gecko_ffi_name)}
</%def>

<%def name="impl_individual_transform(ident, type, gecko_ffi_name)">
    pub fn set_${ident}(&mut self, other: values::computed::${type}) {
        unsafe { self.gecko.${gecko_ffi_name}.clear() };

        if let Some(operation) = other.to_transform_operation() {
            convert_transform(&[operation], &mut self.gecko.${gecko_ffi_name})
        }
    }

    pub fn copy_${ident}_from(&mut self, other: &Self) {
        unsafe { self.gecko.${gecko_ffi_name}.set(&other.gecko.${gecko_ffi_name}); }
    }

    pub fn reset_${ident}(&mut self, other: &Self) {
        self.copy_${ident}_from(other)
    }

    pub fn clone_${ident}(&self) -> values::computed::${type} {
        use values::generics::transform::${type};

        if self.gecko.${gecko_ffi_name}.mRawPtr.is_null() {
            return ${type}::None;
        }

        let list = unsafe { (*self.gecko.${gecko_ffi_name}.to_safe().get()).mHead.as_ref() };

        let mut transform = clone_transform_from_list(list);
        debug_assert_eq!(transform.0.len(), 1);
        ${type}::from_transform_operation(&transform.0.pop().unwrap())
    }
</%def>

<% skip_box_longhands= """display overflow-y vertical-align
                          animation-name animation-delay animation-duration
                          animation-direction animation-fill-mode animation-play-state
                          animation-iteration-count animation-timing-function
                          transition-duration transition-delay
                          transition-timing-function transition-property
                          page-break-before page-break-after rotate
                          scroll-snap-points-x scroll-snap-points-y
                          scroll-snap-type-x scroll-snap-type-y scroll-snap-coordinate
                          perspective-origin -moz-binding will-change
                          overscroll-behavior-x overscroll-behavior-y
                          overflow-clip-box-inline overflow-clip-box-block
                          perspective-origin -moz-binding will-change
                          shape-outside contain touch-action translate
                          scale""" %>
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
                                            gecko_strip_moz_prefix=False) %>

    fn match_display_keyword(
        v: longhands::display::computed_value::T
    ) -> structs::root::mozilla::StyleDisplay {
        use properties::longhands::display::computed_value::T as Keyword;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        match v {
            % for value in display_keyword.values_for('gecko'):
                Keyword::${to_camel_case(value)} =>
                    structs::${display_keyword.gecko_constant(value)},
            % endfor
        }
    }

    pub fn set_display(&mut self, v: longhands::display::computed_value::T) {
        let result = Self::match_display_keyword(v);
        self.gecko.mDisplay = result;
        self.gecko.mOriginalDisplay = result;
    }

    pub fn copy_display_from(&mut self, other: &Self) {
        self.gecko.mDisplay = other.gecko.mDisplay;
        self.gecko.mOriginalDisplay = other.gecko.mDisplay;
    }

    pub fn reset_display(&mut self, other: &Self) {
        self.copy_display_from(other)
    }

    pub fn set_adjusted_display(
        &mut self,
        v: longhands::display::computed_value::T,
        _is_item_or_root: bool
    ) {
        self.gecko.mDisplay = Self::match_display_keyword(v);
    }

    <%call expr="impl_keyword_clone('display', 'mDisplay', display_keyword)"></%call>

    <% overflow_x = data.longhands_by_name["overflow-x"] %>
    pub fn set_overflow_y(&mut self, v: longhands::overflow_y::computed_value::T) {
        use properties::longhands::overflow_x::computed_value::T as BaseType;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        self.gecko.mOverflowY = match v {
            % for value in overflow_x.keyword.values_for('gecko'):
                BaseType::${to_camel_case(value)} => structs::${overflow_x.keyword.gecko_constant(value)} as u8,
            % endfor
        };
    }
    ${impl_simple_copy('overflow_y', 'mOverflowY')}
    pub fn clone_overflow_y(&self) -> longhands::overflow_y::computed_value::T {
        use properties::longhands::overflow_x::computed_value::T as BaseType;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        match self.gecko.mOverflowY as u32 {
            % for value in overflow_x.keyword.values_for('gecko'):
            structs::${overflow_x.keyword.gecko_constant(value)} => BaseType::${to_camel_case(value)},
            % endfor
            x => panic!("Found unexpected value in style struct for overflow_y property: {}", x),
        }
    }

    pub fn set_vertical_align(&mut self, v: longhands::vertical_align::computed_value::T) {
        use values::generics::box_::VerticalAlign;
        let value = match v {
            VerticalAlign::Baseline => structs::NS_STYLE_VERTICAL_ALIGN_BASELINE,
            VerticalAlign::Sub => structs::NS_STYLE_VERTICAL_ALIGN_SUB,
            VerticalAlign::Super => structs::NS_STYLE_VERTICAL_ALIGN_SUPER,
            VerticalAlign::Top => structs::NS_STYLE_VERTICAL_ALIGN_TOP,
            VerticalAlign::TextTop => structs::NS_STYLE_VERTICAL_ALIGN_TEXT_TOP,
            VerticalAlign::Middle => structs::NS_STYLE_VERTICAL_ALIGN_MIDDLE,
            VerticalAlign::Bottom => structs::NS_STYLE_VERTICAL_ALIGN_BOTTOM,
            VerticalAlign::TextBottom => structs::NS_STYLE_VERTICAL_ALIGN_TEXT_BOTTOM,
            VerticalAlign::MozMiddleWithBaseline => {
                structs::NS_STYLE_VERTICAL_ALIGN_MIDDLE_WITH_BASELINE
            },
            VerticalAlign::Length(length) => {
                self.gecko.mVerticalAlign.set(length);
                return;
            },
        };
        self.gecko.mVerticalAlign.set_value(CoordDataValue::Enumerated(value));
    }

    pub fn clone_vertical_align(&self) -> longhands::vertical_align::computed_value::T {
        use values::computed::LengthOrPercentage;
        use values::generics::box_::VerticalAlign;

        let gecko = &self.gecko.mVerticalAlign;
        match gecko.as_value() {
            CoordDataValue::Enumerated(value) => VerticalAlign::from_gecko_keyword(value),
            _ => {
                VerticalAlign::Length(
                    LengthOrPercentage::from_gecko_style_coord(gecko).expect(
                        "expected <length-percentage> for vertical-align",
                    ),
                )
            },
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
            T::Auto   => false,
            T::Always => true,
            T::Avoid  => false,
            T::Left   => true,
            T::Right  => true
        };
        self.gecko.mBreak${kind.title()} = result;
    }

    ${impl_simple_copy('page_break_' + kind, 'mBreak' + kind.title())}

    // Temp fix for Bugzilla bug 24000.
    // See set_page_break_before/after for detail.
    pub fn clone_page_break_${kind}(&self) -> longhands::page_break_${kind}::computed_value::T {
        use computed_values::page_break_${kind}::T;

        if self.gecko.mBreak${kind.title()} { T::Always } else { T::Auto }
    }
    % endfor

    ${impl_style_coord("scroll_snap_points_x", "mScrollSnapPointsX")}
    ${impl_style_coord("scroll_snap_points_y", "mScrollSnapPointsY")}

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

    pub fn reset_scroll_snap_coordinate(&mut self, other: &Self) {
        self.copy_scroll_snap_coordinate_from(other)
    }

    pub fn clone_scroll_snap_coordinate(&self) -> longhands::scroll_snap_coordinate::computed_value::T {
        let vec = self.gecko.mScrollSnapCoordinate.iter().map(|f| f.into()).collect();
        longhands::scroll_snap_coordinate::computed_value::T(vec)
    }

    ${impl_css_url('_moz_binding', 'mBinding')}

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
        use gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_no_properties;

        let v = v.into_iter();

        if v.len() != 0 {
            self.gecko.mTransitions.ensure_len(v.len());
            self.gecko.mTransitionPropertyCount = v.len() as u32;
            for (servo, gecko) in v.zip(self.gecko.mTransitions.iter_mut()) {
                match servo {
                    TransitionProperty::Unsupported(ref ident) => unsafe {
                        Gecko_StyleTransition_SetUnsupportedProperty(gecko, ident.0.as_ptr())
                    },
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
        use gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_all_properties;
        if self.gecko.mTransitionPropertyCount == 1 &&
            self.gecko.mTransitions[0].mProperty == eCSSPropertyExtra_all_properties &&
            self.transition_combined_duration_at(0) <= 0.0f32 {
            return false;
        }

        self.gecko.mTransitionPropertyCount > 0
    }

    pub fn transition_property_at(&self, index: usize)
        -> longhands::transition_property::computed_value::SingleComputedValue {
        use gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_no_properties;
        use gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_variable;
        use gecko_bindings::structs::nsCSSPropertyID::eCSSProperty_UNKNOWN;
        use Atom;

        let property = self.gecko.mTransitions[index].mProperty;
        if property == eCSSProperty_UNKNOWN || property == eCSSPropertyExtra_variable {
            let atom = self.gecko.mTransitions[index].mUnknownProperty.mRawPtr;
            debug_assert!(!atom.is_null());
            TransitionProperty::Unsupported(CustomIdent(unsafe{
                Atom::from_raw(atom)
            }))
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
        self.gecko.mTransitions.ensure_len(other.gecko.mTransitions.len());

        let count = other.gecko.mTransitionPropertyCount;
        self.gecko.mTransitionPropertyCount = count;

        for (index, transition) in self.gecko.mTransitions.iter_mut().enumerate().take(count as usize) {
            transition.mProperty = other.gecko.mTransitions[index].mProperty;
            if transition.mProperty == eCSSProperty_UNKNOWN ||
               transition.mProperty == eCSSPropertyExtra_variable {
                let atom = other.gecko.mTransitions[index].mUnknownProperty.mRawPtr;
                debug_assert!(!atom.is_null());
                unsafe { Gecko_StyleTransition_SetUnsupportedProperty(transition, atom) };
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
        use properties::longhands::animation_name::single_value::SpecifiedValue as AnimationName;
        use Atom;

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
        use values::generics::box_::AnimationIterationCount;

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
        use values::generics::box_::AnimationIterationCount;

        if self.gecko.mAnimations[index].mIterationCount.is_infinite() {
            AnimationIterationCount::Infinite
        } else {
            AnimationIterationCount::Number(self.gecko.mAnimations[index].mIterationCount)
        }
    }

    ${impl_animation_count('iteration_count', 'IterationCount')}
    ${impl_copy_animation_value('iteration_count', 'IterationCount')}

    ${impl_animation_timing_function()}

    <% scroll_snap_type_keyword = Keyword("scroll-snap-type", "None Mandatory Proximity") %>
    ${impl_keyword('scroll_snap_type_y', 'mScrollSnapTypeY', scroll_snap_type_keyword)}
    ${impl_keyword('scroll_snap_type_x', 'mScrollSnapTypeX', scroll_snap_type_keyword)}

    <% overscroll_behavior_keyword = Keyword("overscroll-behavior", "Auto Contain None",
                                             gecko_enum_prefix="StyleOverscrollBehavior") %>
    ${impl_keyword('overscroll_behavior_x', 'mOverscrollBehaviorX', overscroll_behavior_keyword)}
    ${impl_keyword('overscroll_behavior_y', 'mOverscrollBehaviorY', overscroll_behavior_keyword)}

    <% overflow_clip_box_keyword = Keyword("overflow-clip-box", "padding-box content-box") %>
    ${impl_keyword('overflow_clip_box_inline', 'mOverflowClipBoxInline', overflow_clip_box_keyword)}
    ${impl_keyword('overflow_clip_box_block', 'mOverflowClipBoxBlock', overflow_clip_box_keyword)}

    pub fn set_perspective_origin(&mut self, v: longhands::perspective_origin::computed_value::T) {
        self.gecko.mPerspectiveOrigin[0].set(v.horizontal);
        self.gecko.mPerspectiveOrigin[1].set(v.vertical);
    }

    pub fn copy_perspective_origin_from(&mut self, other: &Self) {
        self.gecko.mPerspectiveOrigin[0].copy_from(&other.gecko.mPerspectiveOrigin[0]);
        self.gecko.mPerspectiveOrigin[1].copy_from(&other.gecko.mPerspectiveOrigin[1]);
    }

    pub fn reset_perspective_origin(&mut self, other: &Self) {
        self.copy_perspective_origin_from(other)
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

    ${impl_individual_transform('rotate', 'Rotate', 'mSpecifiedRotate')}
    ${impl_individual_transform('translate', 'Translate', 'mSpecifiedTranslate')}
    ${impl_individual_transform('scale', 'Scale', 'mSpecifiedScale')}

    pub fn set_will_change(&mut self, v: longhands::will_change::computed_value::T) {
        use gecko_bindings::bindings::{Gecko_AppendWillChange, Gecko_ClearWillChange};
        use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_OPACITY;
        use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_SCROLL;
        use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_TRANSFORM;
        use properties::PropertyId;
        use properties::longhands::will_change::computed_value::T;

        fn will_change_bitfield_from_prop_flags(prop: LonghandId) -> u8 {
            use properties::PropertyFlags;
            use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_ABSPOS_CB;
            use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_FIXPOS_CB;
            use gecko_bindings::structs::NS_STYLE_WILL_CHANGE_STACKING_CONTEXT;
            let servo_flags = prop.flags();
            let mut bitfield = 0;

            if servo_flags.contains(PropertyFlags::CREATES_STACKING_CONTEXT) {
                bitfield |= NS_STYLE_WILL_CHANGE_STACKING_CONTEXT;
            }
            if servo_flags.contains(PropertyFlags::FIXPOS_CB) {
                bitfield |= NS_STYLE_WILL_CHANGE_FIXPOS_CB;
            }
            if servo_flags.contains(PropertyFlags::ABSPOS_CB) {
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

                    if let Ok(prop_id) = PropertyId::parse(&feature.0.to_string()) {
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
                                        will_change_bitfield_from_prop_flags(longhand);
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

    pub fn reset_will_change(&mut self, other: &Self) {
        self.copy_will_change_from(other)
    }

    pub fn clone_will_change(&self) -> longhands::will_change::computed_value::T {
        use properties::longhands::will_change::computed_value::T;
        use gecko_bindings::structs::nsAtom;
        use values::CustomIdent;
        use Atom;

        if self.gecko.mWillChange.len() == 0 {
            return T::Auto
        }

        let custom_idents: Vec<CustomIdent> = self.gecko.mWillChange.iter().map(|gecko_atom| {
            unsafe {
                CustomIdent(Atom::from_raw(gecko_atom.mRawPtr as *mut nsAtom))
            }
        }).collect();

        T::AnimateableFeatures(custom_idents.into_boxed_slice())
    }

    <% impl_shape_source("shape_outside", "mShapeOutside") %>

    pub fn set_contain(&mut self, v: longhands::contain::computed_value::T) {
        use gecko_bindings::structs::NS_STYLE_CONTAIN_NONE;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_STRICT;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_LAYOUT;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_STYLE;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_PAINT;
        use gecko_bindings::structs::NS_STYLE_CONTAIN_ALL_BITS;
        use properties::longhands::contain::SpecifiedValue;

        if v.is_empty() {
            self.gecko.mContain = NS_STYLE_CONTAIN_NONE as u8;
            return;
        }

        if v.contains(SpecifiedValue::STRICT) {
            self.gecko.mContain = (NS_STYLE_CONTAIN_STRICT | NS_STYLE_CONTAIN_ALL_BITS) as u8;
            return;
        }

        let mut bitfield = 0;
        if v.contains(SpecifiedValue::LAYOUT) {
            bitfield |= NS_STYLE_CONTAIN_LAYOUT;
        }
        if v.contains(SpecifiedValue::STYLE) {
            bitfield |= NS_STYLE_CONTAIN_STYLE;
        }
        if v.contains(SpecifiedValue::PAINT) {
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
        use properties::longhands::contain::{self, SpecifiedValue};

        let mut servo_flags = contain::computed_value::T::empty();
        let gecko_flags = self.gecko.mContain;

        if gecko_flags & (NS_STYLE_CONTAIN_STRICT as u8) != 0 &&
           gecko_flags & (NS_STYLE_CONTAIN_ALL_BITS as u8) != 0 {
            servo_flags.insert(SpecifiedValue::STRICT | SpecifiedValue::STRICT_BITS);
            return servo_flags;
        }

        if gecko_flags & (NS_STYLE_CONTAIN_LAYOUT as u8) != 0 {
            servo_flags.insert(SpecifiedValue::LAYOUT);
        }
        if gecko_flags & (NS_STYLE_CONTAIN_STYLE as u8) != 0{
            servo_flags.insert(SpecifiedValue::STYLE);
        }
        if gecko_flags & (NS_STYLE_CONTAIN_PAINT as u8) != 0 {
            servo_flags.insert(SpecifiedValue::PAINT);
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
                    Keyword::${to_camel_case(value)} =>
                        structs::${keyword.gecko_constant(value)} ${keyword.maybe_cast('u8')},
                    % endfor
                }
            };
        }
    }

    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
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
                            => Keyword::${to_camel_case(value)},
                        % endfor
                        _ => panic!("Found unexpected value in style struct for ${ident} property"),
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
        use values::specified::background::BackgroundRepeatKeyword;
        use gecko_bindings::structs::nsStyleImageLayers_Repeat;
        use gecko_bindings::structs::StyleImageLayerRepeat;

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
        use properties::longhands::${shorthand}_repeat::single_value::computed_value::T;
        use values::specified::background::BackgroundRepeatKeyword;
        use gecko_bindings::structs::StyleImageLayerRepeat;

        fn to_servo(repeat: StyleImageLayerRepeat) -> BackgroundRepeatKeyword {
            match repeat {
                StyleImageLayerRepeat::Repeat => BackgroundRepeatKeyword::Repeat,
                StyleImageLayerRepeat::Space => BackgroundRepeatKeyword::Space,
                StyleImageLayerRepeat::Round => BackgroundRepeatKeyword::Round,
                StyleImageLayerRepeat::NoRepeat => BackgroundRepeatKeyword::NoRepeat,
                _ => panic!("Found unexpected value in style struct for ${shorthand}_repeat property"),
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

    pub fn reset_${shorthand}_position_${orientation}(&mut self, other: &Self) {
        self.copy_${shorthand}_position_${orientation}_from(other)
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
                debug_assert_eq!(ty, DimensionType::eLengthPercentage as u8);
                value.into()
            }
        }

        longhands::background_size::computed_value::T(
            self.gecko.${image_layers_field}.mLayers.iter().map(|ref layer| {
                if DimensionType::eCover as u8 == layer.mSize.mWidthType {
                    debug_assert_eq!(layer.mSize.mHeightType, DimensionType::eCover as u8);
                    return BackgroundSize::Cover
                }
                if DimensionType::eContain as u8 == layer.mSize.mWidthType {
                    debug_assert_eq!(layer.mSize.mHeightType, DimensionType::eContain as u8);
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
                geckoimage.mImage.set(image)
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
        use gecko_bindings::bindings::Gecko_FillAllImageLayers;
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
                  skip_longhands="list-style-image list-style-type quotes -moz-image-region">

    pub fn set_list_style_image(&mut self, image: longhands::list_style_image::computed_value::T) {
        match image {
            UrlOrNone::None => {
                unsafe {
                    Gecko_SetListStyleImageNone(&mut self.gecko);
                }
            }
            UrlOrNone::Url(ref url) => {
                unsafe {
                    Gecko_SetListStyleImageImageValue(&mut self.gecko, url.image_value.get());
                }
            }
        }
    }

    pub fn copy_list_style_image_from(&mut self, other: &Self) {
        unsafe { Gecko_CopyListStyleImageFrom(&mut self.gecko, &other.gecko); }
    }

    pub fn reset_list_style_image(&mut self, other: &Self) {
        self.copy_list_style_image_from(other)
    }

    pub fn clone_list_style_image(&self) -> longhands::list_style_image::computed_value::T {
        use values::specified::url::SpecifiedImageUrl;

        if self.gecko.mListStyleImage.mRawPtr.is_null() {
            return UrlOrNone::None;
        }

        unsafe {
            let ref gecko_image_request = *self.gecko.mListStyleImage.mRawPtr;
            UrlOrNone::Url(SpecifiedImageUrl::from_image_request(gecko_image_request)
                           .expect("mListStyleImage could not convert to SpecifiedImageUrl"))
        }
    }

    pub fn set_list_style_type(&mut self, v: longhands::list_style_type::computed_value::T, device: &Device) {
        use gecko_bindings::bindings::Gecko_SetCounterStyleToString;
        use nsstring::{nsACString, nsCStr};
        use self::longhands::list_style_type::computed_value::T;
        match v {
            T::CounterStyle(s) => s.to_gecko_value(&mut self.gecko.mCounterStyle, device),
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
        use values::Either;
        use values::generics::CounterStyleOrNone;

        let result = CounterStyleOrNone::from_gecko_value(&self.gecko.mCounterStyle);
        match result {
            Either::First(counter_style) => T::CounterStyle(counter_style),
            Either::Second(string) => T::String(string),
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

        self.gecko.mQuotes.set_move(refptr.get())
    }

    pub fn copy_quotes_from(&mut self, other: &Self) {
        unsafe { self.gecko.mQuotes.set(&other.gecko.mQuotes); }
    }

    pub fn reset_quotes(&mut self, other: &Self) {
        self.copy_quotes_from(other)
    }

    pub fn clone_quotes(&self) -> longhands::quotes::computed_value::T {
        unsafe {
            let ref gecko_quote_values = *self.gecko.mQuotes.mRawPtr;
            longhands::quotes::computed_value::T(
                gecko_quote_values.mQuotePairs.iter().map(|gecko_pair| {
                    (
                        gecko_pair.first.to_string().into_boxed_str(),
                        gecko_pair.second.to_string().into_boxed_str(),
                    )
                }).collect::<Vec<_>>().into_boxed_slice()
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
                self.gecko.mImageRegion.x = rect.left.map(Au::from).unwrap_or(Au(0)).0;
                self.gecko.mImageRegion.y = rect.top.map(Au::from).unwrap_or(Au(0)).0;
                self.gecko.mImageRegion.height = match rect.bottom {
                    Some(value) => (Au::from(value) - Au(self.gecko.mImageRegion.y)).0,
                    None => 0,
                };
                self.gecko.mImageRegion.width = match rect.right {
                    Some(value) => (Au::from(value) - Au(self.gecko.mImageRegion.x)).0,
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

        Either::First(ClipRect {
            top: Some(Au(self.gecko.mImageRegion.y).into()),
            right: Some(Au(self.gecko.mImageRegion.width + self.gecko.mImageRegion.x).into()),
            bottom: Some(Au(self.gecko.mImageRegion.height + self.gecko.mImageRegion.y).into()),
            left: Some(Au(self.gecko.mImageRegion.x).into()),
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

    pub fn reset_box_shadow(&mut self, other: &Self) {
        self.copy_box_shadow_from(other)
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
                    self.gecko.mClip.x = left.to_i32_au();
                } else {
                    self.gecko.mClip.x = 0;
                    self.gecko.mClipFlags |= NS_STYLE_CLIP_LEFT_AUTO as u8;
                }

                if let Some(top) = rect.top {
                    self.gecko.mClip.y = top.to_i32_au();
                } else {
                    self.gecko.mClip.y = 0;
                    self.gecko.mClipFlags |= NS_STYLE_CLIP_TOP_AUTO as u8;
                }

                if let Some(bottom) = rect.bottom {
                    self.gecko.mClip.height = (Au::from(bottom) - Au(self.gecko.mClip.y)).0;
                } else {
                    self.gecko.mClip.height = 1 << 30; // NS_MAXSIZE
                    self.gecko.mClipFlags |= NS_STYLE_CLIP_BOTTOM_AUTO as u8;
                }

                if let Some(right) = rect.right {
                    self.gecko.mClip.width = (Au::from(right) - Au(self.gecko.mClip.x)).0;
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

    pub fn reset_clip(&mut self, other: &Self) {
        self.copy_clip_from(other)
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
                debug_assert_eq!(self.gecko.mClip.x, 0);
                None
            } else {
                Some(Au(self.gecko.mClip.x).into())
            };

            let top = if self.gecko.mClipFlags & NS_STYLE_CLIP_TOP_AUTO as u8 != 0 {
                debug_assert_eq!(self.gecko.mClip.y, 0);
                None
            } else {
                Some(Au(self.gecko.mClip.y).into())
            };

            let bottom = if self.gecko.mClipFlags & NS_STYLE_CLIP_BOTTOM_AUTO as u8 != 0 {
                debug_assert_eq!(self.gecko.mClip.height, 1 << 30); // NS_MAXSIZE
                None
            } else {
                Some(Au(self.gecko.mClip.y + self.gecko.mClip.height).into())
            };

            let right = if self.gecko.mClipFlags & NS_STYLE_CLIP_RIGHT_AUTO as u8 != 0 {
                debug_assert_eq!(self.gecko.mClip.width, 1 << 30); // NS_MAXSIZE
                None
            } else {
                Some(Au(self.gecko.mClip.x + self.gecko.mClip.width).into())
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
                                               CoordDataValue::Factor(factor.0),
                                               gecko_filter),
                % endfor
                Blur(length) => fill_filter(NS_STYLE_FILTER_BLUR,
                                            CoordDataValue::Coord(length.0.to_i32_au()),
                                            gecko_filter),

                HueRotate(angle) => fill_filter(NS_STYLE_FILTER_HUE_ROTATE,
                                                CoordDataValue::from(angle),
                                                gecko_filter),

                DropShadow(shadow) => {
                    gecko_filter.mType = NS_STYLE_FILTER_DROP_SHADOW;

                    fn init_shadow(filter: &mut nsStyleFilter) -> &mut nsCSSShadowArray {
                        unsafe {
                            let ref mut union = filter.__bindgen_anon_1;
                            let shadow_array: &mut *mut nsCSSShadowArray = union.mDropShadow.as_mut();
                            *shadow_array = Gecko_NewCSSShadowArray(1);

                            &mut **shadow_array
                        }
                    }

                    let gecko_shadow = init_shadow(gecko_filter);
                    gecko_shadow.mArray[0].set_from_simple_shadow(shadow);
                },
                Url(ref url) => {
                    unsafe {
                        bindings::Gecko_nsStyleFilter_SetURLValue(gecko_filter, url.url_value.get());
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

    pub fn reset_filter(&mut self, other: &Self) {
        self.copy_filter_from(other)
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
                    filters.push(Filter::Blur(NonNegativeLength::from_gecko_style_coord(
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
            T::AngleWithFlipped(ref orientation, flipped) => {
                unsafe {
                    bindings::Gecko_SetImageOrientation(&mut self.gecko, *orientation as u8, flipped);
                }
            }
        }
    }

    pub fn copy_image_orientation_from(&mut self, other: &Self) {
        unsafe {
            bindings::Gecko_CopyImageOrientationFrom(&mut self.gecko, &other.gecko);
        }
    }

    pub fn reset_image_orientation(&mut self, other: &Self) {
        self.copy_image_orientation_from(other)
    }

    pub fn clone_image_orientation(&self) -> longhands::image_orientation::computed_value::T {
        use gecko_bindings::structs::nsStyleImageOrientation_Angles;
        use properties::longhands::image_orientation::computed_value::T;
        use values::computed::Orientation;

        let gecko_orientation = self.gecko.mImageOrientation.mOrientation;
        if gecko_orientation & structs::nsStyleImageOrientation_Bits_FROM_IMAGE_MASK as u8 != 0 {
            T::FromImage
        } else {
            const ANGLE0: u8 = nsStyleImageOrientation_Angles::ANGLE_0 as u8;
            const ANGLE90: u8 = nsStyleImageOrientation_Angles::ANGLE_90 as u8;
            const ANGLE180: u8 = nsStyleImageOrientation_Angles::ANGLE_180 as u8;
            const ANGLE270: u8 = nsStyleImageOrientation_Angles::ANGLE_270 as u8;

            let flip = gecko_orientation & structs::nsStyleImageOrientation_Bits_FLIP_MASK as u8 != 0;
            let orientation =
                match gecko_orientation & structs::nsStyleImageOrientation_Bits_ORIENTATION_MASK as u8 {
                    ANGLE0 => Orientation::Angle0,
                    ANGLE90 => Orientation::Angle90,
                    ANGLE180 => Orientation::Angle180,
                    ANGLE270 => Orientation::Angle270,
                    _ => unreachable!()
                };
            T::AngleWithFlipped(orientation, flip)
        }
    }
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
                  skip_longhands="text-align text-emphasis-style text-shadow line-height letter-spacing word-spacing
                                  -webkit-text-stroke-width text-emphasis-position -moz-tab-size">

    <% text_align_keyword = Keyword("text-align",
                                    "start end left right center justify -moz-center -moz-left -moz-right char",
                                    gecko_strip_moz_prefix=False) %>
    ${impl_keyword('text_align', 'mTextAlign', text_align_keyword)}

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

    pub fn reset_text_shadow(&mut self, other: &Self) {
        self.copy_text_shadow_from(other)
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
            LineHeight::Length(val) => CoordDataValue::Coord(val.0.to_i32_au()),
            LineHeight::Number(val) => CoordDataValue::Factor(val.0),
            LineHeight::MozBlockHeight =>
                    CoordDataValue::Enumerated(structs::NS_STYLE_LINE_HEIGHT_BLOCK_HEIGHT),
        };
        self.gecko.mLineHeight.set_value(en);
    }

    pub fn clone_line_height(&self) -> longhands::line_height::computed_value::T {
        use values::generics::text::LineHeight;
        return match self.gecko.mLineHeight.as_value() {
            CoordDataValue::Normal => LineHeight::Normal,
            CoordDataValue::Coord(coord) => LineHeight::Length(Au(coord).into()),
            CoordDataValue::Factor(n) => LineHeight::Number(n.into()),
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
        use values::computed::Length;
        use values::generics::text::Spacing;
        debug_assert!(
            matches!(self.gecko.mLetterSpacing.as_value(),
                     CoordDataValue::Normal |
                     CoordDataValue::Coord(_)),
            "Unexpected computed value for letter-spacing");
        Length::from_gecko_style_coord(&self.gecko.mLetterSpacing).map_or(Spacing::Normal, Spacing::Value)
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
        if self.gecko.mTextEmphasisStyle == structs::NS_STYLE_TEXT_EMPHASIS_STYLE_STRING as u8 {
            self.gecko.mTextEmphasisStyleString.truncate();
            self.gecko.mTextEmphasisStyle = structs::NS_STYLE_TEXT_EMPHASIS_STYLE_NONE as u8;
        }
    }

    ${impl_simple_type_with_conversion("text_emphasis_position")}

    pub fn set_text_emphasis_style(&mut self, v: values::computed::TextEmphasisStyle) {
        use values::computed::TextEmphasisStyle;
        use values::specified::text::{TextEmphasisFillMode, TextEmphasisShapeKeyword};

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

    pub fn reset_text_emphasis_style(&mut self, other: &Self) {
        self.copy_text_emphasis_style_from(other)
    }

    pub fn clone_text_emphasis_style(&self) -> values::computed::TextEmphasisStyle {
        use values::computed::TextEmphasisStyle;
        use values::computed::text::TextEmphasisKeywordValue;
        use values::specified::text::{TextEmphasisFillMode, TextEmphasisShapeKeyword};

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

    #[allow(non_snake_case)]
    pub fn set__moz_tab_size(&mut self, v: longhands::_moz_tab_size::computed_value::T) {
        match v {
            MozTabSize::Number(non_negative_number) => {
                self.gecko.mTabSize.set_value(CoordDataValue::Factor(non_negative_number.0));
            }
            MozTabSize::Length(non_negative_length) => {
                self.gecko.mTabSize.set(non_negative_length);
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn clone__moz_tab_size(&self) -> longhands::_moz_tab_size::computed_value::T {
        match self.gecko.mTabSize.as_value() {
            CoordDataValue::Coord(coord) => MozTabSize::Length(Au(coord).into()),
            CoordDataValue::Factor(number) => MozTabSize::Number(From::from(number)),
            _ => unreachable!(),
        }
    }

    <%call expr="impl_coord_copy('_moz_tab_size', 'mTabSize')"></%call>
</%self:impl_trait>

<%self:impl_trait style_struct_name="Text"
                  skip_longhands="text-decoration-line text-overflow initial-letter">

    ${impl_simple_type_with_conversion("text_decoration_line")}

    fn clear_overflow_sides_if_string(&mut self) {
        use gecko_bindings::structs::nsStyleTextOverflowSide;
        fn clear_if_string(side: &mut nsStyleTextOverflowSide) {
            if side.mType == structs::NS_STYLE_TEXT_OVERFLOW_STRING as u8 {
                side.mString.truncate();
                side.mType = structs::NS_STYLE_TEXT_OVERFLOW_CLIP as u8;
            }
        }
        clear_if_string(&mut self.gecko.mTextOverflow.mLeft);
        clear_if_string(&mut self.gecko.mTextOverflow.mRight);
    }

    pub fn set_text_overflow(&mut self, v: longhands::text_overflow::computed_value::T) {
        use gecko_bindings::structs::nsStyleTextOverflowSide;
        use values::specified::text::TextOverflowSide;

        fn set(side: &mut nsStyleTextOverflowSide, value: &TextOverflowSide) {
            let ty = match *value {
                TextOverflowSide::Clip => structs::NS_STYLE_TEXT_OVERFLOW_CLIP,
                TextOverflowSide::Ellipsis => structs::NS_STYLE_TEXT_OVERFLOW_ELLIPSIS,
                TextOverflowSide::String(ref s) => {
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

    pub fn reset_text_overflow(&mut self, other: &Self) {
        self.copy_text_overflow_from(other)
    }

    pub fn clone_text_overflow(&self) -> longhands::text_overflow::computed_value::T {
        use gecko_bindings::structs::nsStyleTextOverflowSide;
        use values::specified::text::TextOverflowSide;

        fn to_servo(side: &nsStyleTextOverflowSide) -> TextOverflowSide {
            match side.mType as u32 {
                structs::NS_STYLE_TEXT_OVERFLOW_CLIP => TextOverflowSide::Clip,
                structs::NS_STYLE_TEXT_OVERFLOW_ELLIPSIS => TextOverflowSide::Ellipsis,
                structs::NS_STYLE_TEXT_OVERFLOW_STRING =>
                    TextOverflowSide::String(side.mString.to_string().into_boxed_str()),
                _ => panic!("Found unexpected value in style struct for text_overflow property"),
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

    pub fn reset_initial_letter(&mut self, other: &Self) {
        self.copy_initial_letter_from(other)
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
            % if ident == "clip_path":
            ShapeSource::ImageOrUrl(ref url) => {
                unsafe {
                    bindings::Gecko_StyleShapeSource_SetURLValue(${ident}, url.url_value.get())
                }
            }
            % elif ident == "shape_outside":
            ShapeSource::ImageOrUrl(image) => {
                unsafe {
                    bindings::Gecko_NewShapeImage(${ident});
                    let style_image = &mut *${ident}.mShapeImage.mPtr;
                    style_image.set(image);
                }
            }
            % else:
               <% raise Exception("Unknown property: %s" % ident) %>
            }
            % endif
            ShapeSource::None => {} // don't change the type
            ShapeSource::Box(reference) => {
                ${ident}.mReferenceBox = reference.into();
                ${ident}.mType = StyleShapeSourceType::Box;
            }
            ShapeSource::Shape(servo_shape, maybe_box) => {
                fn init_shape(${ident}: &mut StyleShapeSource, basic_shape_type: StyleBasicShapeType)
                              -> &mut StyleBasicShape {
                    unsafe {
                        // Create StyleBasicShape in StyleShapeSource. mReferenceBox and mType
                        // will be set manually later.
                        Gecko_NewBasicShape(${ident}, basic_shape_type);
                        &mut *${ident}.mBasicShape.mPtr
                    }
                }
                match servo_shape {
                    BasicShape::Inset(inset) => {
                        let shape = init_shape(${ident}, StyleBasicShapeType::Inset);
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
                        let shape = init_shape(${ident}, StyleBasicShapeType::Circle);
                        unsafe { shape.mCoordinates.set_len(1) };
                        shape.mCoordinates[0].leaky_set_null();
                        circ.radius.to_gecko_style_coord(&mut shape.mCoordinates[0]);

                        shape.mPosition = circ.position.into();
                    }
                    BasicShape::Ellipse(el) => {
                        let shape = init_shape(${ident}, StyleBasicShapeType::Ellipse);
                        unsafe { shape.mCoordinates.set_len(2) };
                        shape.mCoordinates[0].leaky_set_null();
                        el.semiaxis_x.to_gecko_style_coord(&mut shape.mCoordinates[0]);
                        shape.mCoordinates[1].leaky_set_null();
                        el.semiaxis_y.to_gecko_style_coord(&mut shape.mCoordinates[1]);

                        shape.mPosition = el.position.into();
                    }
                    BasicShape::Polygon(poly) => {
                        let shape = init_shape(${ident}, StyleBasicShapeType::Polygon);
                        unsafe {
                            shape.mCoordinates.set_len(poly.coordinates.len() as u32 * 2);
                        }
                        for (i, coord) in poly.coordinates.iter().enumerate() {
                            shape.mCoordinates[2 * i].leaky_set_null();
                            shape.mCoordinates[2 * i + 1].leaky_set_null();
                            coord.0.to_gecko_style_coord(&mut shape.mCoordinates[2 * i]);
                            coord.1.to_gecko_style_coord(&mut shape.mCoordinates[2 * i + 1]);
                        }
                        shape.mFillRule = if poly.fill == FillRule::Evenodd {
                            StyleFillRule::Evenodd
                        } else {
                            StyleFillRule::Nonzero
                        };
                    }
                }

                ${ident}.mReferenceBox = maybe_box.map(Into::into)
                                                  .unwrap_or(StyleGeometryBox::NoBox);
                ${ident}.mType = StyleShapeSourceType::Shape;
            }
        }

    }

    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        (&self.gecko.${gecko_ffi_name}).into()
    }

    pub fn copy_${ident}_from(&mut self, other: &Self) {
        use gecko_bindings::bindings::Gecko_CopyShapeSourceFrom;
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
                  skip_longhands="paint-order stroke-dasharray -moz-context-properties">
    pub fn set_paint_order(&mut self, v: longhands::paint_order::computed_value::T) {
        self.gecko.mPaintOrder = v.0;
    }

    ${impl_simple_copy('paint_order', 'mPaintOrder')}

    pub fn clone_paint_order(&self) -> longhands::paint_order::computed_value::T {
        use properties::longhands::paint_order::computed_value::T;
        T(self.gecko.mPaintOrder)
    }

    pub fn set_stroke_dasharray(&mut self, v: longhands::stroke_dasharray::computed_value::T) {
        use gecko_bindings::structs::nsStyleSVG_STROKE_DASHARRAY_CONTEXT as CONTEXT_VALUE;
        use values::generics::svg::{SVGStrokeDashArray, SvgLengthOrPercentageOrNumber};

        match v {
            SVGStrokeDashArray::Values(v) => {
                let v = v.into_iter();
                self.gecko.mContextFlags &= !CONTEXT_VALUE;
                unsafe {
                    bindings::Gecko_nsStyleSVG_SetDashArrayLength(&mut self.gecko, v.len() as u32);
                }
                for (gecko, servo) in self.gecko.mStrokeDasharray.iter_mut().zip(v) {
                    match servo {
                        SvgLengthOrPercentageOrNumber::LengthOrPercentage(lop) =>
                            gecko.set(lop),
                        SvgLengthOrPercentageOrNumber::Number(num) =>
                            gecko.set_value(CoordDataValue::Factor(num.into())),
                    }
                }
            }
            SVGStrokeDashArray::ContextValue => {
                self.gecko.mContextFlags |= CONTEXT_VALUE;
                unsafe {
                    bindings::Gecko_nsStyleSVG_SetDashArrayLength(&mut self.gecko, 0);
                }
            }
        }
    }

    pub fn copy_stroke_dasharray_from(&mut self, other: &Self) {
        use gecko_bindings::structs::nsStyleSVG_STROKE_DASHARRAY_CONTEXT as CONTEXT_VALUE;
        unsafe {
            bindings::Gecko_nsStyleSVG_CopyDashArray(&mut self.gecko, &other.gecko);
        }
        self.gecko.mContextFlags =
            (self.gecko.mContextFlags & !CONTEXT_VALUE) |
            (other.gecko.mContextFlags & CONTEXT_VALUE);
    }

    pub fn reset_stroke_dasharray(&mut self, other: &Self) {
        self.copy_stroke_dasharray_from(other)
    }

    pub fn clone_stroke_dasharray(&self) -> longhands::stroke_dasharray::computed_value::T {
        use gecko_bindings::structs::nsStyleSVG_STROKE_DASHARRAY_CONTEXT as CONTEXT_VALUE;
        use values::computed::LengthOrPercentage;
        use values::generics::svg::{SVGStrokeDashArray, SvgLengthOrPercentageOrNumber};

        if self.gecko.mContextFlags & CONTEXT_VALUE != 0 {
            debug_assert_eq!(self.gecko.mStrokeDasharray.len(), 0);
            return SVGStrokeDashArray::ContextValue;
        }
        let mut vec = vec![];
        for gecko in self.gecko.mStrokeDasharray.iter() {
            match gecko.as_value() {
                CoordDataValue::Factor(number) =>
                    vec.push(SvgLengthOrPercentageOrNumber::Number(number.into())),
                CoordDataValue::Coord(coord) =>
                    vec.push(SvgLengthOrPercentageOrNumber::LengthOrPercentage(
                        LengthOrPercentage::Length(Au(coord).into()).into())),
                CoordDataValue::Percent(p) =>
                    vec.push(SvgLengthOrPercentageOrNumber::LengthOrPercentage(
                        LengthOrPercentage::Percentage(Percentage(p)).into())),
                CoordDataValue::Calc(calc) =>
                    vec.push(SvgLengthOrPercentageOrNumber::LengthOrPercentage(
                        LengthOrPercentage::Calc(calc.into()).into())),
                _ => unreachable!(),
            }
        }
        SVGStrokeDashArray::Values(vec)
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
        for (gecko, servo) in self.gecko.mContextProps.iter_mut().zip(v) {
            if (servo.0).0 == atom!("fill") {
                self.gecko.mContextPropsBits |= structs::NS_STYLE_CONTEXT_PROPERTY_FILL as u8;
            } else if (servo.0).0 == atom!("stroke") {
                self.gecko.mContextPropsBits |= structs::NS_STYLE_CONTEXT_PROPERTY_STROKE as u8;
            } else if (servo.0).0 == atom!("fill-opacity") {
                self.gecko.mContextPropsBits |= structs::NS_STYLE_CONTEXT_PROPERTY_FILL_OPACITY as u8;
            } else if (servo.0).0 == atom!("stroke-opacity") {
                self.gecko.mContextPropsBits |= structs::NS_STYLE_CONTEXT_PROPERTY_STROKE_OPACITY as u8;
            }
            gecko.mRawPtr = (servo.0).0.into_addrefed();
        }
    }

    #[allow(non_snake_case)]
    pub fn copy__moz_context_properties_from(&mut self, other: &Self) {
        unsafe {
            bindings::Gecko_nsStyleSVG_CopyContextProperties(&mut self.gecko, &other.gecko);
        }
    }

    #[allow(non_snake_case)]
    pub fn reset__moz_context_properties(&mut self, other: &Self) {
        self.copy__moz_context_properties_from(other)
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

<%self:impl_trait style_struct_name="InheritedUI"
                  skip_longhands="cursor caret-color">
    pub fn set_cursor(&mut self, v: longhands::cursor::computed_value::T) {
        use style_traits::cursor::CursorKind;

        self.gecko.mCursor = match v.keyword {
            CursorKind::Auto => structs::NS_STYLE_CURSOR_AUTO,
            CursorKind::None => structs::NS_STYLE_CURSOR_NONE,
            CursorKind::Default => structs::NS_STYLE_CURSOR_DEFAULT,
            CursorKind::Pointer => structs::NS_STYLE_CURSOR_POINTER,
            CursorKind::ContextMenu => structs::NS_STYLE_CURSOR_CONTEXT_MENU,
            CursorKind::Help => structs::NS_STYLE_CURSOR_HELP,
            CursorKind::Progress => structs::NS_STYLE_CURSOR_SPINNING,
            CursorKind::Wait => structs::NS_STYLE_CURSOR_WAIT,
            CursorKind::Cell => structs::NS_STYLE_CURSOR_CELL,
            CursorKind::Crosshair => structs::NS_STYLE_CURSOR_CROSSHAIR,
            CursorKind::Text => structs::NS_STYLE_CURSOR_TEXT,
            CursorKind::VerticalText => structs::NS_STYLE_CURSOR_VERTICAL_TEXT,
            CursorKind::Alias => structs::NS_STYLE_CURSOR_ALIAS,
            CursorKind::Copy => structs::NS_STYLE_CURSOR_COPY,
            CursorKind::Move => structs::NS_STYLE_CURSOR_MOVE,
            CursorKind::NoDrop => structs::NS_STYLE_CURSOR_NO_DROP,
            CursorKind::NotAllowed => structs::NS_STYLE_CURSOR_NOT_ALLOWED,
            CursorKind::Grab => structs::NS_STYLE_CURSOR_GRAB,
            CursorKind::Grabbing => structs::NS_STYLE_CURSOR_GRABBING,
            CursorKind::EResize => structs::NS_STYLE_CURSOR_E_RESIZE,
            CursorKind::NResize => structs::NS_STYLE_CURSOR_N_RESIZE,
            CursorKind::NeResize => structs::NS_STYLE_CURSOR_NE_RESIZE,
            CursorKind::NwResize => structs::NS_STYLE_CURSOR_NW_RESIZE,
            CursorKind::SResize => structs::NS_STYLE_CURSOR_S_RESIZE,
            CursorKind::SeResize => structs::NS_STYLE_CURSOR_SE_RESIZE,
            CursorKind::SwResize => structs::NS_STYLE_CURSOR_SW_RESIZE,
            CursorKind::WResize => structs::NS_STYLE_CURSOR_W_RESIZE,
            CursorKind::EwResize => structs::NS_STYLE_CURSOR_EW_RESIZE,
            CursorKind::NsResize => structs::NS_STYLE_CURSOR_NS_RESIZE,
            CursorKind::NeswResize => structs::NS_STYLE_CURSOR_NESW_RESIZE,
            CursorKind::NwseResize => structs::NS_STYLE_CURSOR_NWSE_RESIZE,
            CursorKind::ColResize => structs::NS_STYLE_CURSOR_COL_RESIZE,
            CursorKind::RowResize => structs::NS_STYLE_CURSOR_ROW_RESIZE,
            CursorKind::AllScroll => structs::NS_STYLE_CURSOR_ALL_SCROLL,
            CursorKind::ZoomIn => structs::NS_STYLE_CURSOR_ZOOM_IN,
            CursorKind::ZoomOut => structs::NS_STYLE_CURSOR_ZOOM_OUT,
            // note: the following properties are gecko-only.
            CursorKind::MozGrab => structs::NS_STYLE_CURSOR_GRAB,
            CursorKind::MozGrabbing => structs::NS_STYLE_CURSOR_GRABBING,
            CursorKind::MozZoomIn => structs::NS_STYLE_CURSOR_ZOOM_IN,
            CursorKind::MozZoomOut => structs::NS_STYLE_CURSOR_ZOOM_OUT,
        } as u8;

        unsafe {
            Gecko_SetCursorArrayLength(&mut self.gecko, v.images.len());
        }
        for i in 0..v.images.len() {
            unsafe {
                Gecko_SetCursorImageValue(
                    &mut self.gecko.mCursorImages[i],
                    v.images[i].url.image_value.get(),
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
            Gecko_CopyCursorArrayFrom(&mut self.gecko, &other.gecko);
        }
    }

    pub fn reset_cursor(&mut self, other: &Self) {
        self.copy_cursor_from(other)
    }

    pub fn clone_cursor(&self) -> longhands::cursor::computed_value::T {
        use values::computed::ui::CursorImage;
        use style_traits::cursor::CursorKind;
        use values::specified::url::SpecifiedImageUrl;

        let keyword = match self.gecko.mCursor as u32 {
            structs::NS_STYLE_CURSOR_AUTO => CursorKind::Auto,
            structs::NS_STYLE_CURSOR_NONE => CursorKind::None,
            structs::NS_STYLE_CURSOR_DEFAULT => CursorKind::Default,
            structs::NS_STYLE_CURSOR_POINTER => CursorKind::Pointer,
            structs::NS_STYLE_CURSOR_CONTEXT_MENU => CursorKind::ContextMenu,
            structs::NS_STYLE_CURSOR_HELP => CursorKind::Help,
            structs::NS_STYLE_CURSOR_SPINNING => CursorKind::Progress,
            structs::NS_STYLE_CURSOR_WAIT => CursorKind::Wait,
            structs::NS_STYLE_CURSOR_CELL => CursorKind::Cell,
            structs::NS_STYLE_CURSOR_CROSSHAIR => CursorKind::Crosshair,
            structs::NS_STYLE_CURSOR_TEXT => CursorKind::Text,
            structs::NS_STYLE_CURSOR_VERTICAL_TEXT => CursorKind::VerticalText,
            structs::NS_STYLE_CURSOR_ALIAS => CursorKind::Alias,
            structs::NS_STYLE_CURSOR_COPY => CursorKind::Copy,
            structs::NS_STYLE_CURSOR_MOVE => CursorKind::Move,
            structs::NS_STYLE_CURSOR_NO_DROP => CursorKind::NoDrop,
            structs::NS_STYLE_CURSOR_NOT_ALLOWED => CursorKind::NotAllowed,
            structs::NS_STYLE_CURSOR_GRAB => CursorKind::Grab,
            structs::NS_STYLE_CURSOR_GRABBING => CursorKind::Grabbing,
            structs::NS_STYLE_CURSOR_E_RESIZE => CursorKind::EResize,
            structs::NS_STYLE_CURSOR_N_RESIZE => CursorKind::NResize,
            structs::NS_STYLE_CURSOR_NE_RESIZE => CursorKind::NeResize,
            structs::NS_STYLE_CURSOR_NW_RESIZE => CursorKind::NwResize,
            structs::NS_STYLE_CURSOR_S_RESIZE => CursorKind::SResize,
            structs::NS_STYLE_CURSOR_SE_RESIZE => CursorKind::SeResize,
            structs::NS_STYLE_CURSOR_SW_RESIZE => CursorKind::SwResize,
            structs::NS_STYLE_CURSOR_W_RESIZE => CursorKind::WResize,
            structs::NS_STYLE_CURSOR_EW_RESIZE => CursorKind::EwResize,
            structs::NS_STYLE_CURSOR_NS_RESIZE => CursorKind::NsResize,
            structs::NS_STYLE_CURSOR_NESW_RESIZE => CursorKind::NeswResize,
            structs::NS_STYLE_CURSOR_NWSE_RESIZE => CursorKind::NwseResize,
            structs::NS_STYLE_CURSOR_COL_RESIZE => CursorKind::ColResize,
            structs::NS_STYLE_CURSOR_ROW_RESIZE => CursorKind::RowResize,
            structs::NS_STYLE_CURSOR_ALL_SCROLL => CursorKind::AllScroll,
            structs::NS_STYLE_CURSOR_ZOOM_IN => CursorKind::ZoomIn,
            structs::NS_STYLE_CURSOR_ZOOM_OUT => CursorKind::ZoomOut,
            _ => panic!("Found unexpected value in style struct for cursor property"),
        };

        let images = self.gecko.mCursorImages.iter().map(|gecko_cursor_image| {
            let url = unsafe {
                let gecko_image_request = gecko_cursor_image.mImage.mRawPtr.as_ref().unwrap();
                SpecifiedImageUrl::from_image_request(&gecko_image_request)
                    .expect("mCursorImages.mImage could not convert to SpecifiedImageUrl")
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

    <%call expr="impl_color('caret_color', 'mCaretColor')"></%call>
</%self:impl_trait>

<%self:impl_trait style_struct_name="Column"
                  skip_longhands="column-count column-rule-width">

    #[allow(unused_unsafe)]
    pub fn set_column_count(&mut self, v: longhands::column_count::computed_value::T) {
        use gecko_bindings::structs::{NS_STYLE_COLUMN_COUNT_AUTO, nsStyleColumn_kMaxColumnCount};

        self.gecko.mColumnCount = match v {
            ColumnCount::Integer(integer) => {
                cmp::min(integer.0 as u32, unsafe { nsStyleColumn_kMaxColumnCount })
            },
            ColumnCount::Auto => NS_STYLE_COLUMN_COUNT_AUTO
        };
    }

    ${impl_simple_copy('column_count', 'mColumnCount')}

    pub fn clone_column_count(&self) -> longhands::column_count::computed_value::T {
        use gecko_bindings::structs::{NS_STYLE_COLUMN_COUNT_AUTO, nsStyleColumn_kMaxColumnCount};
        if self.gecko.mColumnCount != NS_STYLE_COLUMN_COUNT_AUTO {
            debug_assert!(self.gecko.mColumnCount >= 1 &&
                          self.gecko.mColumnCount <= nsStyleColumn_kMaxColumnCount);
            ColumnCount::Integer((self.gecko.mColumnCount as i32).into())
        } else {
            ColumnCount::Auto
        }
    }

    <% impl_non_negative_length("column_rule_width", "mColumnRuleWidth",
                                round_to_pixels=True) %>
</%self:impl_trait>

<%self:impl_trait style_struct_name="Counters"
                  skip_longhands="content counter-increment counter-reset">
    pub fn ineffective_content_property(&self) -> bool {
        self.gecko.mContents.is_empty()
    }

    pub fn set_content(&mut self, v: longhands::content::computed_value::T, device: &Device) {
        use values::CustomIdent;
        use values::computed::counters::{Content, ContentItem};
        use values::generics::CounterStyleOrNone;
        use gecko_bindings::structs::nsStyleContentData;
        use gecko_bindings::structs::nsStyleContentAttr;
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

        fn set_counter_function(
            data: &mut nsStyleContentData,
            content_type: nsStyleContentType,
            name: &CustomIdent,
            sep: &str,
            style: CounterStyleOrNone,
            device: &Device,
        ) {
            debug_assert!(content_type == eStyleContentType_Counter ||
                          content_type == eStyleContentType_Counters);
            let counter_func = unsafe {
                bindings::Gecko_SetCounterFunction(data, content_type).as_mut().unwrap()
            };
            counter_func.mIdent.assign(name.0.as_slice());
            if content_type == eStyleContentType_Counters {
                counter_func.mSeparator.assign_utf8(sep);
            }
            style.to_gecko_value(&mut counter_func.mCounterStyle, device);
        }

        match v {
            Content::None |
            Content::Normal => {
                // Ensure destructors run, otherwise we could leak.
                if !self.gecko.mContents.is_empty() {
                    unsafe {
                        Gecko_ClearAndResizeStyleContents(&mut self.gecko, 0);
                    }
                }
            },
            Content::MozAltContent => {
                unsafe {
                    Gecko_ClearAndResizeStyleContents(&mut self.gecko, 1);
                    *self.gecko.mContents[0].mContent.mString.as_mut() = ptr::null_mut();
                }
                self.gecko.mContents[0].mType = eStyleContentType_AltContent;
            },
            Content::Items(items) => {
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
                    match *item {
                        ContentItem::String(ref value) => {
                            self.gecko.mContents[i].mType = eStyleContentType_String;
                            unsafe {
                                // NB: we share allocators, so doing this is fine.
                                *self.gecko.mContents[i].mContent.mString.as_mut() =
                                    as_utf16_and_forget(&value);
                            }
                        }
                        ContentItem::Attr(ref attr) => {
                            self.gecko.mContents[i].mType = eStyleContentType_Attr;
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
                            => self.gecko.mContents[i].mType = eStyleContentType_OpenQuote,
                        ContentItem::CloseQuote
                            => self.gecko.mContents[i].mType = eStyleContentType_CloseQuote,
                        ContentItem::NoOpenQuote
                            => self.gecko.mContents[i].mType = eStyleContentType_NoOpenQuote,
                        ContentItem::NoCloseQuote
                            => self.gecko.mContents[i].mType = eStyleContentType_NoCloseQuote,
                        ContentItem::Counter(ref name, ref style) => {
                            set_counter_function(
                                &mut self.gecko.mContents[i],
                                eStyleContentType_Counter,
                                &name,
                                "",
                                style.clone(),
                                device,
                            );
                        }
                        ContentItem::Counters(ref name, ref sep, ref style) => {
                            set_counter_function(
                                &mut self.gecko.mContents[i],
                                eStyleContentType_Counters,
                                &name,
                                &sep,
                                style.clone(),
                                device,
                            );
                        }
                        ContentItem::Url(ref url) => {
                            unsafe {
                                bindings::Gecko_SetContentDataImageValue(
                                    &mut self.gecko.mContents[i],
                                    url.image_value.get(),
                                )
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

    pub fn reset_content(&mut self, other: &Self) {
        self.copy_content_from(other)
    }

    pub fn clone_content(&self) -> longhands::content::computed_value::T {
        use {Atom, Namespace};
        use gecko::conversions::string_from_chars_pointer;
        use gecko_bindings::structs::nsStyleContentType::*;
        use values::computed::counters::{Content, ContentItem};
        use values::{CustomIdent, Either};
        use values::generics::CounterStyleOrNone;
        use values::specified::url::SpecifiedImageUrl;
        use values::specified::Attr;

        if self.gecko.mContents.is_empty() {
            return Content::Normal;
        }

        if self.gecko.mContents.len() == 1 &&
           self.gecko.mContents[0].mType == eStyleContentType_AltContent {
            return Content::MozAltContent;
        }

        Content::Items(
            self.gecko.mContents.iter().map(|gecko_content| {
                match gecko_content.mType {
                    eStyleContentType_OpenQuote => ContentItem::OpenQuote,
                    eStyleContentType_CloseQuote => ContentItem::CloseQuote,
                    eStyleContentType_NoOpenQuote => ContentItem::NoOpenQuote,
                    eStyleContentType_NoCloseQuote => ContentItem::NoCloseQuote,
                    eStyleContentType_String => {
                        let gecko_chars = unsafe { gecko_content.mContent.mString.as_ref() };
                        let string = unsafe { string_from_chars_pointer(*gecko_chars) };
                        ContentItem::String(string.into_boxed_str())
                    },
                    eStyleContentType_Attr => {
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
                    eStyleContentType_Counter | eStyleContentType_Counters => {
                        let gecko_function =
                            unsafe { &**gecko_content.mContent.mCounters.as_ref() };
                        let ident = CustomIdent(Atom::from(&*gecko_function.mIdent));
                        let style =
                            CounterStyleOrNone::from_gecko_value(&gecko_function.mCounterStyle);
                        let style = match style {
                            Either::First(counter_style) => counter_style,
                            Either::Second(_) =>
                                unreachable!("counter function shouldn't have single string type"),
                        };
                        if gecko_content.mType == eStyleContentType_Counter {
                            ContentItem::Counter(ident, style)
                        } else {
                            let separator = gecko_function.mSeparator.to_string();
                            ContentItem::Counters(ident, separator.into_boxed_str(), style)
                        }
                    },
                    eStyleContentType_Image => {
                        unsafe {
                            let gecko_image_request =
                                &**gecko_content.mContent.mImage.as_ref();
                            ContentItem::Url(
                                SpecifiedImageUrl::from_image_request(gecko_image_request)
                                    .expect("mContent could not convert to SpecifiedImageUrl")
                            )
                        }
                    },
                    _ => panic!("Found unexpected value in style struct for content property"),
                }
            }).collect::<Vec<_>>().into_boxed_slice()
        )
    }

    % for counter_property in ["Increment", "Reset"]:
        pub fn set_counter_${counter_property.lower()}(
            &mut self,
            v: longhands::counter_${counter_property.lower()}::computed_value::T
        ) {
            unsafe {
                bindings::Gecko_ClearAndResizeCounter${counter_property}s(&mut self.gecko, v.len() as u32);
                for (i, ref pair) in v.iter().enumerate() {
                    self.gecko.m${counter_property}s[i].mCounter.assign(pair.name.0.as_slice());
                    self.gecko.m${counter_property}s[i].mValue = pair.value;
                }
            }
        }

        pub fn copy_counter_${counter_property.lower()}_from(&mut self, other: &Self) {
            unsafe {
                bindings::Gecko_CopyCounter${counter_property}sFrom(&mut self.gecko, &other.gecko)
            }
        }

        pub fn reset_counter_${counter_property.lower()}(&mut self, other: &Self) {
            self.copy_counter_${counter_property.lower()}_from(other)
        }

        pub fn clone_counter_${counter_property.lower()}(
            &self
        ) -> longhands::counter_${counter_property.lower()}::computed_value::T {
            use values::generics::counters::CounterPair;
            use values::CustomIdent;
            use gecko_string_cache::Atom;

            longhands::counter_${counter_property.lower()}::computed_value::T::new(
                self.gecko.m${counter_property}s.iter().map(|ref gecko_counter| {
                    CounterPair {
                        name: CustomIdent(Atom::from(gecko_counter.mCounter.to_string())),
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
