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
use cssparser::Color;
use custom_properties::ComputedValuesMap;
use gecko_bindings::bindings;
% for style_struct in data.style_structs:
use gecko_bindings::structs::${style_struct.gecko_ffi_name};
use gecko_bindings::bindings::Gecko_Construct_Default_${style_struct.gecko_ffi_name};
use gecko_bindings::bindings::Gecko_CopyConstruct_${style_struct.gecko_ffi_name};
use gecko_bindings::bindings::Gecko_Destroy_${style_struct.gecko_ffi_name};
% endfor
use gecko_bindings::bindings::Gecko_Construct_nsStyleVariables;
use gecko_bindings::bindings::Gecko_CopyCursorArrayFrom;
use gecko_bindings::bindings::Gecko_CopyFontFamilyFrom;
use gecko_bindings::bindings::Gecko_CopyImageValueFrom;
use gecko_bindings::bindings::Gecko_CopyListStyleImageFrom;
use gecko_bindings::bindings::Gecko_CopyListStyleTypeFrom;
use gecko_bindings::bindings::Gecko_Destroy_nsStyleVariables;
use gecko_bindings::bindings::Gecko_EnsureImageLayersLength;
use gecko_bindings::bindings::Gecko_FontFamilyList_AppendGeneric;
use gecko_bindings::bindings::Gecko_FontFamilyList_AppendNamed;
use gecko_bindings::bindings::Gecko_FontFamilyList_Clear;
use gecko_bindings::bindings::Gecko_SetCursorArrayLength;
use gecko_bindings::bindings::Gecko_SetCursorImage;
use gecko_bindings::bindings::Gecko_NewCSSShadowArray;
use gecko_bindings::bindings::Gecko_nsStyleFont_SetLang;
use gecko_bindings::bindings::Gecko_nsStyleFont_CopyLangFrom;
use gecko_bindings::bindings::Gecko_SetListStyleImage;
use gecko_bindings::bindings::Gecko_SetListStyleImageNone;
use gecko_bindings::bindings::Gecko_SetListStyleType;
use gecko_bindings::bindings::Gecko_SetNullImageValue;
use gecko_bindings::bindings::ServoComputedValuesBorrowedOrNull;
use gecko_bindings::bindings::{Gecko_ResetFilters, Gecko_CopyFiltersFrom};
use gecko_bindings::bindings::RawGeckoPresContextBorrowed;
use gecko_bindings::structs;
use gecko_bindings::structs::nsStyleVariables;
use gecko_bindings::sugar::ns_style_coord::{CoordDataValue, CoordData, CoordDataMut};
use gecko_bindings::sugar::ownership::HasArcFFI;
use gecko::values::convert_nscolor_to_rgba;
use gecko::values::convert_rgba_to_nscolor;
use gecko::values::GeckoStyleCoordConvertible;
use gecko::values::round_border_to_device_pixels;
use logical_geometry::WritingMode;
use properties::longhands;
use properties::{DeclaredValue, Importance, LonghandId};
use properties::{PropertyDeclaration, PropertyDeclarationBlock, PropertyDeclarationId};
use std::fmt::{self, Debug};
use std::mem::{forget, transmute, zeroed};
use std::ptr;
use std::sync::Arc;
use std::cmp;
use values::computed::ToComputedValue;
use values::{Either, Auto};
use computed_values::border_style;

pub mod style_structs {
    % for style_struct in data.style_structs:
    pub use super::${style_struct.gecko_struct_name} as ${style_struct.name};
    % endfor
}

#[derive(Clone, Debug)]
pub struct ComputedValues {
    % for style_struct in data.style_structs:
    ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
    % endfor

    custom_properties: Option<Arc<ComputedValuesMap>>,
    shareable: bool,
    pub writing_mode: WritingMode,
    pub root_font_size: Au,
}

impl ComputedValues {
    pub fn inherit_from(parent: &Arc<Self>, default: &Arc<Self>) -> Arc<Self> {
        Arc::new(ComputedValues {
            custom_properties: parent.custom_properties.clone(),
            shareable: parent.shareable,
            writing_mode: parent.writing_mode,
            root_font_size: parent.root_font_size,
            % for style_struct in data.style_structs:
            % if style_struct.inherited:
            ${style_struct.ident}: parent.${style_struct.ident}.clone(),
            % else:
            ${style_struct.ident}: default.${style_struct.ident}.clone(),
            % endif
            % endfor
        })
    }

    pub fn new(custom_properties: Option<Arc<ComputedValuesMap>>,
           shareable: bool,
           writing_mode: WritingMode,
           root_font_size: Au,
            % for style_struct in data.style_structs:
           ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
            % endfor
    ) -> Self {
        ComputedValues {
            custom_properties: custom_properties,
            shareable: shareable,
            writing_mode: writing_mode,
            root_font_size: root_font_size,
            % for style_struct in data.style_structs:
            ${style_struct.ident}: ${style_struct.ident},
            % endfor
        }
    }

    pub fn default_values(pres_context: RawGeckoPresContextBorrowed) -> Arc<Self> {
        Arc::new(ComputedValues {
            custom_properties: None,
            shareable: true,
            writing_mode: WritingMode::empty(), // FIXME(bz): This seems dubious
            root_font_size: longhands::font_size::get_initial_value(), // FIXME(bz): Also seems dubious?
            % for style_struct in data.style_structs:
                ${style_struct.ident}: style_structs::${style_struct.name}::default(pres_context),
            % endfor
        })
    }

    #[inline]
    pub fn is_display_contents(&self) -> bool {
        self.get_box().clone_display() == longhands::display::computed_value::T::contents
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
    #[inline]
    pub fn mutate_${style_struct.name_lower}(&mut self) -> &mut style_structs::${style_struct.name} {
        Arc::make_mut(&mut self.${style_struct.ident})
    }
    % endfor

    pub fn custom_properties(&self) -> Option<Arc<ComputedValuesMap>> {
        self.custom_properties.as_ref().map(|x| x.clone())
    }

    #[allow(non_snake_case)]
    pub fn has_moz_binding(&self) -> bool {
        !self.get_box().gecko.mBinding.mRawPtr.is_null()
    }

    // FIXME(bholley): Implement this properly.
    #[inline]
    pub fn is_multicol(&self) -> bool { false }

    pub fn to_declaration_block(&self, property: PropertyDeclarationId) -> PropertyDeclarationBlock {
        match property {
            % for prop in data.longhands:
                % if prop.animatable:
                    PropertyDeclarationId::Longhand(LonghandId::${prop.camel_case}) => {
                        PropertyDeclarationBlock {
                            declarations: vec![
                                (PropertyDeclaration::${prop.camel_case}(DeclaredValue::Value(
                                    % if prop.boxed:
                                        Box::new(
                                    % endif
                                    longhands::${prop.ident}::SpecifiedValue::from_computed_value(
                                      &self.get_${prop.style_struct.ident.strip("_")}().clone_${prop.ident}())
                                    % if prop.boxed:
                                        )
                                    % endif

                                 )),
                                 Importance::Normal)
                            ],
                            important_count: 0
                        }
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

<%def name="impl_keyword_setter(ident, gecko_ffi_name, keyword, cast_type='u8')">
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
    }
</%def>

<%def name="impl_keyword_clone(ident, gecko_ffi_name, keyword)">
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use properties::longhands::${ident}::computed_value::T as Keyword;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        match ${get_gecko_property(gecko_ffi_name)} ${keyword.maybe_cast("u32")} {
            % for value in keyword.values_for('gecko'):
            structs::${keyword.gecko_constant(value)} => Keyword::${to_rust_ident(value)},
            % endfor
            % if keyword.gecko_inexhaustive:
            x => panic!("Found unexpected value in style struct for ${ident} property: {:?}", x),
            % endif
        }
    }
</%def>


/// Convert a Servo color into an nscolor; with currentColor as 0
///
/// Call sites will need to be updated after https://bugzilla.mozilla.org/show_bug.cgi?id=760345
fn color_to_nscolor_zero_currentcolor(color: Color) -> structs::nscolor {
    match color {
        Color::RGBA(rgba) => {
            convert_rgba_to_nscolor(&rgba)
        },
        Color::CurrentColor => 0,
    }
}

<%def name="impl_color_setter(ident, gecko_ffi_name, complex_color=True)">
    #[allow(unreachable_code)]
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        % if complex_color:
            let result = v.into();
        % else:
            let result = color_to_nscolor_zero_currentcolor(v);
        % endif
        ${set_gecko_property(gecko_ffi_name, "result")}
    }
</%def>

<%def name="impl_color_copy(ident, gecko_ffi_name, complex_color=True)">
    #[allow(non_snake_case)]
    pub fn copy_${ident}_from(&mut self, other: &Self) {
        let color = ${get_gecko_property(gecko_ffi_name, self_param = "other")};
        ${set_gecko_property(gecko_ffi_name, "color")};
    }
</%def>

<%def name="impl_color_clone(ident, gecko_ffi_name, complex_color=True)">
    #[allow(non_snake_case)]
    pub fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        % if complex_color:
            ${get_gecko_property(gecko_ffi_name)}.into()
        % else:
            Color::RGBA(convert_nscolor_to_rgba(${get_gecko_property(gecko_ffi_name)}))
        % endif
    }
</%def>

<%def name="impl_keyword(ident, gecko_ffi_name, keyword, need_clone, **kwargs)">
<%call expr="impl_keyword_setter(ident, gecko_ffi_name, keyword, **kwargs)"></%call>
<%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
%if need_clone:
<%call expr="impl_keyword_clone(ident, gecko_ffi_name, keyword)"></%call>
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
            use values::computed::Position;
            Position {
                horizontal: self.gecko.${gecko_ffi_name}.mXPosition.into(),
                vertical: self.gecko.${gecko_ffi_name}.mYPosition.into(),
            }
        }
    % endif
</%def>

<%def name="impl_color(ident, gecko_ffi_name, need_clone=False, complex_color=True)">
<%call expr="impl_color_setter(ident, gecko_ffi_name, complex_color)"></%call>
<%call expr="impl_color_copy(ident, gecko_ffi_name, complex_color)"></%call>
% if need_clone:
    <%call expr="impl_color_clone(ident, gecko_ffi_name, complex_color)"></%call>
% endif
</%def>

<%def name="impl_svg_paint(ident, gecko_ffi_name, need_clone=False, complex_color=True)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, mut v: longhands::${ident}::computed_value::T) {
        use values::computed::SVGPaintKind;
        use self::structs::nsStyleSVGPaintType;

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
                    if let Some(ffi) = url.for_ffi() {
                        bindings::Gecko_nsStyleSVGPaint_SetURLValue(paint, ffi);
                    } else {
                        return;
                    }
                }
            }
            SVGPaintKind::Color(color) => {
                paint.mType = nsStyleSVGPaintType::eStyleSVGPaintType_Color;
                unsafe {
                    *paint.mPaint.mColor.as_mut() = color_to_nscolor_zero_currentcolor(color);
                }
            }
        }

        if let Some(fallback) = fallback {
            paint.mFallbackColor = color_to_nscolor_zero_currentcolor(fallback);
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
</%def>

<%def name="impl_app_units(ident, gecko_ffi_name, need_clone, round_to_pixels=False)">
    #[allow(non_snake_case)]
    pub fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        % if round_to_pixels:
        let au_per_device_px = Au(self.gecko.mTwipsPerPixel);
        self.gecko.${gecko_ffi_name} = round_border_to_device_pixels(v, au_per_device_px).0;
        % else:
        self.gecko.${gecko_ffi_name} = v.0;
        % endif
    }
<%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
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

<%def name="impl_corner_style_coord(ident, gecko_ffi_name, x_index, y_index, need_clone=False)">
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
            use properties::longhands::${ident}::computed_value::T;
            use euclid::Size2D;
            let width = GeckoStyleCoordConvertible::from_gecko_style_coord(
                            &self.gecko.${gecko_ffi_name}.data_at(${x_index}))
                            .expect("Failed to clone ${ident}");
            let height = GeckoStyleCoordConvertible::from_gecko_style_coord(
                            &self.gecko.${gecko_ffi_name}.data_at(${y_index}))
                            .expect("Failed to clone ${ident}");
            T(Size2D::new(width, height))
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
                    if let Some(ffi) = url.for_ffi() {
                        let ptr = bindings::Gecko_NewURLValue(ffi);
                        RefPtr::from_addrefed(ptr)
                    } else {
                        self.gecko.${gecko_ffi_name}.clear();
                        return;
                    }
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
        <% raise Exception("Do not know how to handle clone ") %>
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

<%def name="raw_impl_trait(style_struct, skip_longhands='', skip_additionals='')">
<%
    longhands = [x for x in style_struct.longhands
                if not (skip_longhands == "*" or x.name in skip_longhands.split())]

    #
    # Make a list of types we can't auto-generate.
    #
    force_stub = [];
    # These live in an nsFont member in Gecko. Should be straightforward to do manually.
    force_stub += ["font-variant"]
    # These have unusual representations in gecko.
    force_stub += ["list-style-type"]

    # These are part of shorthands so we must include them in stylo builds,
    # but we haven't implemented the stylo glue for the longhand
    # so we generate a stub
    force_stub += ["flex-basis", # position

                   # transition
                   "transition-duration", "transition-timing-function",
                   "transition-property", "transition-delay",
                   ]

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
        "MaxLength": impl_style_coord,
        "MinLength": impl_style_coord,
        "Number": impl_simple,
        "Opacity": impl_simple,
        "CSSColor": impl_color,
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
            if longhand.predefined_type in ["CSSColor"]:
                args.update(complex_color=longhand.complex_color)

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
<%self:impl_trait style_struct_name="Border"
                  skip_longhands="${skip_border_longhands} border-image-source border-image-outset
                                  border-image-repeat border-image-width border-image-slice"
                  skip_additionals="*">

    % for side in SIDES:
    <% impl_keyword("border_%s_style" % side.ident, "mBorderStyle[%s]" % side.index, border_style_keyword,
                    need_clone=True) %>

    <% impl_color("border_%s_color" % side.ident, "(mBorderColor)[%s]" % side.index, need_clone=True) %>

    <% impl_app_units("border_%s_width" % side.ident, "mComputedBorder.%s" % side.ident, need_clone=True,
                      round_to_pixels=True) %>

    pub fn border_${side.ident}_has_nonzero_width(&self) -> bool {
        self.gecko.mComputedBorder.${side.ident} != 0
    }
    % endfor

    % for corner in CORNERS:
    <% impl_corner_style_coord("border_%s_radius" % corner.ident,
                               "mBorderRadius",
                               corner.x_index,
                               corner.y_index,
                               need_clone=True) %>
    % endfor

    pub fn set_border_image_source(&mut self, v: longhands::border_image_source::computed_value::T) {
        unsafe {
            // Prevent leaking of the last elements we did set
            Gecko_SetNullImageValue(&mut self.gecko.mBorderImageSource);
        }

        if let Some(image) = v.0 {
            // TODO: We need to make border-image-source match with background-image
            // until then we are setting with_url to false
            self.gecko.mBorderImageSource.set(image, false, &mut false)
        }
    }

    pub fn copy_border_image_source_from(&mut self, other: &Self) {
        unsafe {
            Gecko_CopyImageValueFrom(&mut self.gecko.mBorderImageSource,
                                     &other.gecko.mBorderImageSource);
        }
    }

    pub fn set_border_image_outset(&mut self, v: longhands::border_image_outset::computed_value::T) {
        % for side in SIDES:
            v.${side.index}.to_gecko_style_coord(&mut self.gecko.mBorderImageOutset
                                                          .data_at_mut(${side.index}));
        % endfor
    }

    pub fn copy_border_image_outset_from(&mut self, other: &Self) {
        % for side in SIDES:
            self.gecko.mBorderImageOutset.data_at_mut(${side.index})
                .copy_from(&other.gecko.mBorderImageOutset.data_at(${side.index}));
        % endfor
    }

    pub fn set_border_image_repeat(&mut self, v: longhands::border_image_repeat::computed_value::T) {
        use properties::longhands::border_image_repeat::computed_value::RepeatKeyword;
        use gecko_bindings::structs::{NS_STYLE_BORDER_IMAGE_REPEAT_STRETCH, NS_STYLE_BORDER_IMAGE_REPEAT_REPEAT};
        use gecko_bindings::structs::{NS_STYLE_BORDER_IMAGE_REPEAT_ROUND, NS_STYLE_BORDER_IMAGE_REPEAT_SPACE};

        % for i, side in enumerate(["H", "V"]):
            let k = match v.${i} {
                RepeatKeyword::Stretch => NS_STYLE_BORDER_IMAGE_REPEAT_STRETCH,
                RepeatKeyword::Repeat => NS_STYLE_BORDER_IMAGE_REPEAT_REPEAT,
                RepeatKeyword::Round => NS_STYLE_BORDER_IMAGE_REPEAT_ROUND,
                RepeatKeyword::Space => NS_STYLE_BORDER_IMAGE_REPEAT_SPACE,
            };

            self.gecko.mBorderImageRepeat${side} = k as u8;
        % endfor
    }

    pub fn copy_border_image_repeat_from(&mut self, other: &Self) {
        self.gecko.mBorderImageRepeatH = other.gecko.mBorderImageRepeatH;
        self.gecko.mBorderImageRepeatV = other.gecko.mBorderImageRepeatV;
    }

    pub fn set_border_image_width(&mut self, v: longhands::border_image_width::computed_value::T) {
        use properties::longhands::border_image_width::computed_value::SingleComputedValue;

        % for side in SIDES:
        match v.${side.index} {
            SingleComputedValue::Auto => {
                self.gecko.mBorderImageWidth.data_at_mut(${side.index}).set_value(CoordDataValue::Auto)
            },
            SingleComputedValue::LengthOrPercentage(l) => {
                l.to_gecko_style_coord(&mut self.gecko.mBorderImageWidth.data_at_mut(${side.index}))
            },
            SingleComputedValue::Number(n) => {
                self.gecko.mBorderImageWidth.data_at_mut(${side.index}).set_value(CoordDataValue::Factor(n))
            },
        }
        % endfor
    }

    pub fn copy_border_image_width_from(&mut self, other: &Self) {
        % for side in SIDES:
            self.gecko.mBorderImageWidth.data_at_mut(${side.index})
                .copy_from(&other.gecko.mBorderImageWidth.data_at(${side.index}));
        % endfor
    }

    pub fn set_border_image_slice(&mut self, v: longhands::border_image_slice::computed_value::T) {
        use gecko_bindings::structs::{NS_STYLE_BORDER_IMAGE_SLICE_NOFILL, NS_STYLE_BORDER_IMAGE_SLICE_FILL};
        use properties::longhands::border_image_slice::computed_value::PercentageOrNumber;

        for (i, corner) in v.corners.iter().enumerate() {
            match *corner {
                PercentageOrNumber::Percentage(p) => {
                    self.gecko.mBorderImageSlice.data_at_mut(i).set_value(CoordDataValue::Percent(p.0))
                },
                PercentageOrNumber::Number(n) => {
                    self.gecko.mBorderImageSlice.data_at_mut(i).set_value(CoordDataValue::Factor(n))
                },
            }
        }

        let fill = if v.fill {
            NS_STYLE_BORDER_IMAGE_SLICE_FILL
        } else {
            NS_STYLE_BORDER_IMAGE_SLICE_NOFILL
        };
        self.gecko.mBorderImageFill = fill as u8;
    }

    pub fn copy_border_image_slice_from(&mut self, other: &Self) {
        for i in 0..4 {
            self.gecko.mBorderImageSlice.data_at_mut(i)
                .copy_from(&other.gecko.mBorderImageSlice.data_at(i));
        }
        self.gecko.mBorderImageFill = other.gecko.mBorderImageFill;
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
                  skip_longhands="${skip_position_longhands} z-index box-sizing order align-content
                                  justify-content align-self justify-self align-items
                                  justify-items grid-auto-rows grid-auto-columns">
    % for side in SIDES:
    <% impl_split_style_coord("%s" % side.ident,
                              "mOffset",
                              side.index,
                              need_clone=True) %>
    % endfor

    pub fn set_z_index(&mut self, v: longhands::z_index::computed_value::T) {
        use properties::longhands::z_index::computed_value::T;
        match v {
            T::Auto => self.gecko.mZIndex.set_value(CoordDataValue::Auto),
            T::Number(n) => self.gecko.mZIndex.set_value(CoordDataValue::Integer(n)),
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
        use properties::longhands::z_index::computed_value::T;
        return match self.gecko.mZIndex.as_value() {
            CoordDataValue::Auto => T::Auto,
            CoordDataValue::Integer(n) => T::Number(n),
            _ => {
                debug_assert!(false);
                T::Number(0)
            }
        }
    }

    pub fn set_align_content(&mut self, v: longhands::align_content::computed_value::T) {
        self.gecko.mAlignContent = v.bits()
    }

    ${impl_simple_copy('align_content', 'mAlignContent')}

    pub fn set_justify_content(&mut self, v: longhands::justify_content::computed_value::T) {
        self.gecko.mJustifyContent = v.bits()
    }

    ${impl_simple_copy('justify_content', 'mJustifyContent')}

    pub fn set_align_self(&mut self, v: longhands::align_self::computed_value::T) {
        self.gecko.mAlignSelf = v.0.bits()
    }

    ${impl_simple_copy('align_self', 'mAlignSelf')}

    pub fn set_justify_self(&mut self, v: longhands::justify_self::computed_value::T) {
        self.gecko.mJustifySelf = v.0.bits()
    }

    ${impl_simple_copy('justify_self', 'mJustifySelf')}

    pub fn set_align_items(&mut self, v: longhands::align_items::computed_value::T) {
        self.gecko.mAlignItems = v.0.bits()
    }

    ${impl_simple_copy('align_items', 'mAlignItems')}

    pub fn set_justify_items(&mut self, v: longhands::justify_items::computed_value::T) {
        self.gecko.mJustifyItems = v.0.bits()
    }

    ${impl_simple_copy('justify_items', 'mJustifyItems')}

    pub fn clone_justify_items(&self) -> longhands::justify_items::computed_value::T {
        use values::specified::align::{AlignFlags, JustifyItems};
        JustifyItems(AlignFlags::from_bits(self.gecko.mJustifyItems)
                                          .expect("mJustifyItems contains valid flags"))
    }

    pub fn set_box_sizing(&mut self, v: longhands::box_sizing::computed_value::T) {
        use computed_values::box_sizing::T;
        use gecko_bindings::structs::StyleBoxSizing;
        // TODO: guess what to do with box-sizing: padding-box
        self.gecko.mBoxSizing = match v {
            T::content_box => StyleBoxSizing::Content,
            T::border_box => StyleBoxSizing::Border
        }
    }
    ${impl_simple_copy('box_sizing', 'mBoxSizing')}

    pub fn set_order(&mut self, v: longhands::order::computed_value::T) {
        self.gecko.mOrder = v;
    }

    pub fn clone_order(&self) -> longhands::order::computed_value::T {
        self.gecko.mOrder
    }

    ${impl_simple_copy('order', 'mOrder')}

    % for value in GRID_LINES:
    pub fn set_${value.name}(&mut self, v: longhands::${value.name}::computed_value::T) {
        use nsstring::nsCString;
        use gecko_bindings::structs::{nsStyleGridLine_kMinLine, nsStyleGridLine_kMaxLine};

        let ident = v.ident.unwrap_or(String::new());
        self.gecko.${value.gecko}.mLineName.assign_utf8(&nsCString::from(&*ident));
        self.gecko.${value.gecko}.mHasSpan = v.is_span;
        self.gecko.${value.gecko}.mInteger = v.integer.map(|i| {
            // clamping the integer between a range
            cmp::max(nsStyleGridLine_kMinLine, cmp::min(i, nsStyleGridLine_kMaxLine))
        }).unwrap_or(0);
    }

    pub fn copy_${value.name}_from(&mut self, other: &Self) {
        self.gecko.${value.gecko}.mHasSpan = other.gecko.${value.gecko}.mHasSpan;
        self.gecko.${value.gecko}.mInteger = other.gecko.${value.gecko}.mInteger;
        self.gecko.${value.gecko}.mLineName.assign(&*other.gecko.${value.gecko}.mLineName);
    }
    % endfor

    % for kind in ["rows", "columns"]:
    pub fn set_grid_auto_${kind}(&mut self, v: longhands::grid_auto_rows::computed_value::T) {
        use values::specified::grid::TrackSize;

        match v {
            TrackSize::FitContent(lop) => {
                // Gecko sets min value to None and max value to the actual value in fit-content
                // https://dxr.mozilla.org/mozilla-central/rev/0eef1d5/layout/style/nsRuleNode.cpp#8221
                self.gecko.mGridAuto${kind.title()}Min.set_value(CoordDataValue::None);
                lop.to_gecko_style_coord(&mut self.gecko.mGridAuto${kind.title()}Max);
            },
            TrackSize::Breadth(breadth) => {
                // Set the value to both fields if there's one breadth value
                // https://dxr.mozilla.org/mozilla-central/rev/0eef1d5/layout/style/nsRuleNode.cpp#8230
                breadth.to_gecko_style_coord(&mut self.gecko.mGridAuto${kind.title()}Min);
                breadth.to_gecko_style_coord(&mut self.gecko.mGridAuto${kind.title()}Max);
            },
            TrackSize::MinMax(min, max) => {
                min.to_gecko_style_coord(&mut self.gecko.mGridAuto${kind.title()}Min);
                max.to_gecko_style_coord(&mut self.gecko.mGridAuto${kind.title()}Max);
            },
        }
    }

    pub fn copy_grid_auto_${kind}_from(&mut self, other: &Self) {
        self.gecko.mGridAuto${kind.title()}Min.copy_from(&other.gecko.mGridAuto${kind.title()}Min);
        self.gecko.mGridAuto${kind.title()}Max.copy_from(&other.gecko.mGridAuto${kind.title()}Max);
    }
    % endfor

</%self:impl_trait>

<% skip_outline_longhands = " ".join("outline-style outline-width".split() +
                                     ["-moz-outline-radius-{0}".format(x.ident.replace("_", ""))
                                      for x in CORNERS]) %>
<%self:impl_trait style_struct_name="Outline"
                  skip_longhands="${skip_outline_longhands}"
                  skip_additionals="*">

    #[allow(non_snake_case)]
    pub fn set_outline_style(&mut self, v: longhands::outline_style::computed_value::T) {
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        let result = match v {
            % for value in border_style_keyword.values_for('gecko'):
                Either::Second(border_style::T::${to_rust_ident(value)}) =>
                    structs::${border_style_keyword.gecko_constant(value)} ${border_style_keyword.maybe_cast("u8")},
            % endfor
                Either::First(Auto) =>
                    structs::${border_style_keyword.gecko_constant('auto')} ${border_style_keyword.maybe_cast("u8")},
        };
        ${set_gecko_property("mOutlineStyle", "result")}
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

    <% impl_app_units("outline_width", "mActualOutlineWidth", need_clone=True,
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

<%self:impl_trait style_struct_name="Font"
    skip_longhands="font-family font-size font-size-adjust font-weight font-synthesis -x-lang"
    skip_additionals="*">

    pub fn set_font_family(&mut self, v: longhands::font_family::computed_value::T) {
        use properties::longhands::font_family::computed_value::FontFamily;
        use gecko_bindings::structs::FontFamilyType;

        let list = &mut self.gecko.mFont.fontlist;
        unsafe { Gecko_FontFamilyList_Clear(list); }

        for family in &v.0 {
            match *family {
                FontFamily::FamilyName(ref name) => {
                    unsafe { Gecko_FontFamilyList_AppendNamed(list, name.0.as_ptr()); }
                }
                FontFamily::Generic(ref name) => {
                    let family_type =
                        if name == &atom!("serif") { FontFamilyType::eFamily_serif }
                        else if name == &atom!("sans-serif") { FontFamilyType::eFamily_sans_serif }
                        else if name == &atom!("cursive") { FontFamilyType::eFamily_cursive }
                        else if name == &atom!("fantasy") { FontFamilyType::eFamily_fantasy }
                        else if name == &atom!("monospace") { FontFamilyType::eFamily_monospace }
                        else { panic!("Unknown generic font family") };
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
    }

    // FIXME(bholley): Gecko has two different sizes, one of which (mSize) is the
    // actual computed size, and the other of which (mFont.size) is the 'display
    // size' which takes font zooming into account. We don't handle font zooming yet.
    pub fn set_font_size(&mut self, v: longhands::font_size::computed_value::T) {
        self.gecko.mFont.size = v.0;
        self.gecko.mSize = v.0;
    }
    pub fn copy_font_size_from(&mut self, other: &Self) {
        self.gecko.mFont.size = other.gecko.mFont.size;
        self.gecko.mSize = other.gecko.mSize;
    }
    pub fn clone_font_size(&self) -> longhands::font_size::computed_value::T {
        Au(self.gecko.mSize)
    }

    pub fn set_font_weight(&mut self, v: longhands::font_weight::computed_value::T) {
        self.gecko.mFont.weight = v as u16;
    }
    ${impl_simple_copy('font_weight', 'mFont.weight')}

    pub fn clone_font_weight(&self) -> longhands::font_weight::computed_value::T {
        debug_assert!(self.gecko.mFont.weight >= 100);
        debug_assert!(self.gecko.mFont.weight <= 900);
        debug_assert!(self.gecko.mFont.weight % 10 == 0);
        unsafe { transmute(self.gecko.mFont.weight) }
    }

    pub fn set_font_synthesis(&mut self, v: longhands::font_synthesis::computed_value::T) {
        use gecko_bindings::structs::{NS_FONT_SYNTHESIS_WEIGHT, NS_FONT_SYNTHESIS_STYLE};

        self.gecko.mFont.synthesis = 0;
        if v.weight {
            self.gecko.mFont.synthesis |= NS_FONT_SYNTHESIS_WEIGHT as u8;
        }
        if v.style {
            self.gecko.mFont.synthesis |= NS_FONT_SYNTHESIS_STYLE as u8;
        }
    }

    pub fn copy_font_synthesis_from(&mut self, other: &Self) {
        self.gecko.mFont.synthesis = other.gecko.mFont.synthesis;
    }

    pub fn set_font_size_adjust(&mut self, v: longhands::font_size_adjust::computed_value::T) {
        use properties::longhands::font_size_adjust::computed_value::T;
        match v {
            T::None => self.gecko.mFont.sizeAdjust = -1.0 as f32,
            T::Number(n) => self.gecko.mFont.sizeAdjust = n.0 as f32,
        }
    }

    pub fn copy_font_size_adjust_from(&mut self, other: &Self) {
        self.gecko.mFont.sizeAdjust = other.gecko.mFont.sizeAdjust;
    }

    pub fn clone_font_size_adjust(&self) -> longhands::font_size_adjust::computed_value::T {
        use properties::longhands::font_size_adjust::computed_value::T;
        use values::specified::Number;

        match self.gecko.mFont.sizeAdjust {
            -1.0 => T::None,
            _ => T::Number(Number(self.gecko.mFont.sizeAdjust)),
        }
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
</%self:impl_trait>

<%def name="impl_copy_animation_value(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn copy_animation_${ident}_from(&mut self, other: &Self) {
        unsafe { self.gecko.mAnimations.ensure_len(other.gecko.mAnimations.len()) };

        let count = other.gecko.mAnimation${gecko_ffi_name}Count;
        self.gecko.mAnimation${gecko_ffi_name}Count = count;

        // The length of mAnimations is often greater than mAnimationXXCount,
        // don't copy values over the count.
        for (index, animation) in self.gecko.mAnimations.iter_mut().enumerate().take(count as usize) {
            animation.m${gecko_ffi_name} = other.gecko.mAnimations[index].m${gecko_ffi_name};
        }
    }
</%def>

<%def name="impl_animation_count(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn animation_${ident}_count(&self) -> usize {
        self.gecko.mAnimation${gecko_ffi_name}Count as usize
    }
</%def>

<%def name="impl_animation_time_value(ident, gecko_ffi_name)">
    #[allow(non_snake_case)]
    pub fn set_animation_${ident}(&mut self, v: longhands::animation_${ident}::computed_value::T) {
        assert!(v.0.len() > 0);
        unsafe { self.gecko.mAnimations.ensure_len(v.0.len()) };

        self.gecko.mAnimation${gecko_ffi_name}Count = v.0.len() as u32;
        for (servo, gecko) in v.0.into_iter().zip(self.gecko.mAnimations.iter_mut()) {
            gecko.m${gecko_ffi_name} = servo.seconds() * 1000.;
        }
    }
    #[allow(non_snake_case)]
    pub fn animation_${ident}_at(&self, index: usize)
        -> longhands::animation_${ident}::computed_value::SingleComputedValue {
        use values::specified::Time;
        Time(self.gecko.mAnimations[index].m${gecko_ffi_name} / 1000.)
    }
    ${impl_animation_count(ident, gecko_ffi_name)}
    ${impl_copy_animation_value(ident, gecko_ffi_name)}
</%def>

<%def name="impl_animation_keyword(ident, gecko_ffi_name, keyword, cast_type='u8')">
    #[allow(non_snake_case)]
    pub fn set_animation_${ident}(&mut self, v: longhands::animation_${ident}::computed_value::T) {
        use properties::longhands::animation_${ident}::single_value::computed_value::T as Keyword;
        use gecko_bindings::structs;

        assert!(v.0.len() > 0);
        unsafe { self.gecko.mAnimations.ensure_len(v.0.len()) };

        self.gecko.mAnimation${gecko_ffi_name}Count = v.0.len() as u32;

        for (servo, gecko) in v.0.into_iter().zip(self.gecko.mAnimations.iter_mut()) {
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
                          page-break-before page-break-after
                          scroll-snap-points-x scroll-snap-points-y transform
                          scroll-snap-type-y scroll-snap-coordinate
                          perspective-origin transform-origin""" %>
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
    pub fn set_adjusted_display(&mut self, v: longhands::display::computed_value::T) {
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

    // overflow-y is implemented as a newtype of overflow-x, so we need special handling.
    // We could generalize this if we run into other newtype keywords.
    <% overflow_x = data.longhands_by_name["overflow-x"] %>
    pub fn set_overflow_y(&mut self, v: longhands::overflow_y::computed_value::T) {
        use properties::longhands::overflow_x::computed_value::T as BaseType;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        self.gecko.mOverflowY = match v.0 {
            % for value in overflow_x.keyword.values_for('gecko'):
                BaseType::${to_rust_ident(value)} => structs::${overflow_x.keyword.gecko_constant(value)} as u8,
            % endfor
        };
    }
    ${impl_simple_copy('overflow_y', 'mOverflowY')}
    pub fn clone_overflow_y(&self) -> longhands::overflow_y::computed_value::T {
        use properties::longhands::overflow_x::computed_value::T as BaseType;
        use properties::longhands::overflow_y::computed_value::T as NewType;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        match self.gecko.mOverflowY as u32 {
            % for value in overflow_x.keyword.values_for('gecko'):
            structs::${overflow_x.keyword.gecko_constant(value)} => NewType(BaseType::${to_rust_ident(value)}),
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

    // Temp fix for Bugzilla bug 24000.
    // Map 'auto' and 'avoid' to false, and 'always', 'left', and 'right' to true.
    // "A conforming user agent may interpret the values 'left' and 'right'
    // as 'always'." - CSS2.1, section 13.3.1
    pub fn set_page_break_before(&mut self, v: longhands::page_break_before::computed_value::T) {
        use computed_values::page_break_before::T;
        let result = match v {
            T::auto   => false,
            T::always => true,
            T::avoid  => false,
            T::left   => true,
            T::right  => true
        };
        self.gecko.mBreakBefore = result;
    }

    ${impl_simple_copy('page_break_before', 'mBreakBefore')}

    // Temp fix for Bugzilla bug 24000.
    // See set_page_break_before for detail.
    pub fn set_page_break_after(&mut self, v: longhands::page_break_after::computed_value::T) {
        use computed_values::page_break_after::T;
        let result = match v {
            T::auto   => false,
            T::always => true,
            T::avoid  => false,
            T::left   => true,
            T::right  => true
        };
        self.gecko.mBreakAfter = result;
    }

    ${impl_simple_copy('page_break_after', 'mBreakAfter')}

    pub fn set_scroll_snap_points_x(&mut self, v: longhands::scroll_snap_points_x::computed_value::T) {
        match v.0 {
            None => self.gecko.mScrollSnapPointsX.set_value(CoordDataValue::None),
            Some(l) => l.to_gecko_style_coord(&mut self.gecko.mScrollSnapPointsX),
        };
    }

    ${impl_coord_copy('scroll_snap_points_x', 'mScrollSnapPointsX')}

    pub fn set_scroll_snap_points_y(&mut self, v: longhands::scroll_snap_points_y::computed_value::T) {
        match v.0 {
            None => self.gecko.mScrollSnapPointsY.set_value(CoordDataValue::None),
            Some(l) => l.to_gecko_style_coord(&mut self.gecko.mScrollSnapPointsY),
        };
    }

    ${impl_coord_copy('scroll_snap_points_y', 'mScrollSnapPointsY')}

    pub fn set_scroll_snap_coordinate(&mut self, v: longhands::scroll_snap_coordinate::computed_value::T) {
        unsafe { self.gecko.mScrollSnapCoordinate.set_len_pod(v.0.len() as u32); }
        for (gecko, servo) in self.gecko.mScrollSnapCoordinate
                               .iter_mut()
                               .zip(v.0.iter()) {
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

    <%def name="transform_function_arm(name, keyword, items)">
        <%
            pattern = None
            if name == "matrix":
                # m11, m12, m13, ..
                indices = [str(i) + str(j) for i in range(1, 5) for j in range(1, 5)]
                # m11: number1, m12: number2, ..
                single_patterns = ["m%s: number%s" % (index, i + 1) for (i, index) in enumerate(indices)]
                pattern = "ComputedMatrix { %s }" % ", ".join(single_patterns)
            else:
                # Generate contents of pattern from items
                pattern = ", ".join([b + str(a+1) for (a,b) in enumerate(items)])

            # First %s substituted with the call to GetArrayItem, the second
            # %s substituted with the corresponding variable
            css_value_setters = {
                "length" : "bindings::Gecko_CSSValue_SetAbsoluteLength(%s, %s.0)",
                "percentage" : "bindings::Gecko_CSSValue_SetPercentage(%s, %s)",
                "lop" : "%s.set_lop(%s)",
                "angle" : "bindings::Gecko_CSSValue_SetAngle(%s, %s.0)",
                "number" : "bindings::Gecko_CSSValue_SetNumber(%s, %s)",
            }
        %>
        longhands::transform::computed_value::ComputedOperation::${name.title()}(${pattern}) => {
            bindings::Gecko_CSSValue_SetFunction(gecko_value, ${len(items) + 1});
            bindings::Gecko_CSSValue_SetKeyword(
                bindings::Gecko_CSSValue_GetArrayItem(gecko_value, 0),
                eCSSKeyword_${keyword}
            );
            % for index, item in enumerate(items):
                ${css_value_setters[item] % (
                    "bindings::Gecko_CSSValue_GetArrayItem(gecko_value, %d)" % (index + 1),
                    item + str(index + 1)
                )};
            % endfor
        }
    </%def>
    pub fn convert_transform(input: Vec<longhands::transform::computed_value::ComputedOperation>,
                             output: &mut structs::root::RefPtr<structs::root::nsCSSValueSharedList>) {
        use gecko_bindings::structs::nsCSSKeyword::*;
        use gecko_bindings::sugar::refptr::RefPtr;
        use properties::longhands::transform::computed_value::ComputedMatrix;

        unsafe { output.clear() };

        let list = unsafe {
            RefPtr::from_addrefed(bindings::Gecko_NewCSSValueSharedList(input.len() as u32))
        };

        let mut cur = list.mHead;
        let mut iter = input.into_iter();
        while !cur.is_null() {
            let gecko_value = unsafe { &mut (*cur).mValue };
            let servo = iter.next().expect("Gecko_NewCSSValueSharedList should create a shared \
                                            value list of the same length as the transform vector");
            unsafe {
                match servo {
                    ${transform_function_arm("matrix", "matrix3d", ["number"] * 16)}
                    ${transform_function_arm("skew", "skew", ["angle"] * 2)}
                    ${transform_function_arm("translate", "translate3d", ["lop", "lop", "length"])}
                    ${transform_function_arm("scale", "scale3d", ["number"] * 3)}
                    ${transform_function_arm("rotate", "rotate3d", ["number"] * 3 + ["angle"])}
                    ${transform_function_arm("perspective", "perspective", ["length"])}
                }
                cur = (*cur).mNext;
            }
        }
        debug_assert!(iter.next().is_none());
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
        Self::convert_transform(vec, &mut self.gecko.mSpecifiedTransform);
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
                "angle" : "Angle(bindings::Gecko_CSSValue_GetAngle(%s))",
                "number" : "bindings::Gecko_CSSValue_GetNumber(%s)",
            }
        %>
        eCSSKeyword_${keyword} => {
            ComputedOperation::${name.title()}(
            % if name == "matrix":
                ComputedMatrix {
            % endif
            % for index, item in enumerate(items):
                % if name == "matrix":
                    m${index / 4 + 1}${index % 4 + 1}:
                % endif
                ${css_value_getters[item] % (
                    "bindings::Gecko_CSSValue_GetArrayItemConst(gecko_value, %d)" % (index + 1)
                )},
            % endfor
            % if name == "matrix":
                }
            % endif
            )
        },
    </%def>
    pub fn clone_transform(&self) -> longhands::transform::computed_value::T {
        use app_units::Au;
        use gecko_bindings::structs::nsCSSKeyword::*;
        use properties::longhands::transform::computed_value;
        use properties::longhands::transform::computed_value::ComputedMatrix;
        use properties::longhands::transform::computed_value::ComputedOperation;
        use values::computed::Angle;

        if self.gecko.mSpecifiedTransform.mRawPtr.is_null() {
            return computed_value::T(None);
        }

        let mut result = vec![];
        let mut cur = unsafe { (*self.gecko.mSpecifiedTransform.to_safe().get()).mHead };
        while !cur.is_null() {
            let gecko_value = unsafe { &(*cur).mValue };
            let transform_function = unsafe {
                bindings::Gecko_CSSValue_GetKeyword(bindings::Gecko_CSSValue_GetArrayItemConst(gecko_value, 0))
            };
            let servo = unsafe {
                match transform_function {
                    ${computed_operation_arm("matrix", "matrix3d", ["number"] * 16)}
                    ${computed_operation_arm("skew", "skew", ["angle"] * 2)}
                    ${computed_operation_arm("translate", "translate3d", ["lop", "lop", "length"])}
                    ${computed_operation_arm("scale", "scale3d", ["number"] * 3)}
                    ${computed_operation_arm("rotate", "rotate3d", ["number"] * 3 + ["angle"])}
                    ${computed_operation_arm("perspective", "perspective", ["length"])}
                    _ => panic!("We shouldn't set any other transform function types"),
                }
            };
            result.push(servo);
            unsafe { cur = (&*cur).mNext };
        }
        computed_value::T(Some(result))
    }

    pub fn set_animation_name(&mut self, v: longhands::animation_name::computed_value::T) {
        use nsstring::nsCString;
        unsafe { self.gecko.mAnimations.ensure_len(v.0.len()) };

        if v.0.len() > 0 {
            self.gecko.mAnimationNameCount = v.0.len() as u32;
            for (servo, gecko) in v.0.into_iter().zip(self.gecko.mAnimations.iter_mut()) {
                gecko.mName.assign_utf8(&nsCString::from(servo.0.to_string()));
            }
        } else {
            unsafe { self.gecko.mAnimations[0].mName.truncate(); }
        }
    }
    pub fn animation_name_at(&self, index: usize)
        -> longhands::animation_name::computed_value::SingleComputedValue {
        use Atom;
        use properties::longhands::animation_name::single_value::SpecifiedValue as AnimationName;
        // XXX: Is there any effective ways?
        AnimationName(Atom::from(String::from_utf16_lossy(&self.gecko.mAnimations[index].mName[..])))
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

    pub fn set_animation_iteration_count(&mut self, v: longhands::animation_iteration_count::computed_value::T) {
        use std::f32;
        use properties::longhands::animation_iteration_count::single_value::SpecifiedValue as AnimationIterationCount;

        assert!(v.0.len() > 0);
        unsafe { self.gecko.mAnimations.ensure_len(v.0.len()) };

        self.gecko.mAnimationIterationCountCount = v.0.len() as u32;
        for (servo, gecko) in v.0.into_iter().zip(self.gecko.mAnimations.iter_mut()) {
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

    pub fn set_animation_timing_function(&mut self, v: longhands::animation_timing_function::computed_value::T) {
        assert!(v.0.len() > 0);
        unsafe { self.gecko.mAnimations.ensure_len(v.0.len()) };

        self.gecko.mAnimationTimingFunctionCount = v.0.len() as u32;
        for (servo, gecko) in v.0.into_iter().zip(self.gecko.mAnimations.iter_mut()) {
            gecko.mTimingFunction = servo.into();
        }
    }
    ${impl_animation_count('timing_function', 'TimingFunction')}
    ${impl_copy_animation_value('timing_function', 'TimingFunction')}
    pub fn animation_timing_function_at(&self, index: usize)
        -> longhands::animation_timing_function::computed_value::SingleComputedValue {
        self.gecko.mAnimations[index].mTimingFunction.into()
    }

    <% scroll_snap_type_keyword = Keyword("scroll-snap-type", "none mandatory proximity") %>

    ${impl_keyword('scroll_snap_type_y', 'mScrollSnapTypeY', scroll_snap_type_keyword, need_clone=False)}

    pub fn set_perspective_origin(&mut self, v: longhands::perspective_origin::computed_value::T) {
        self.gecko.mPerspectiveOrigin[0].set(v.horizontal);
        self.gecko.mPerspectiveOrigin[1].set(v.vertical);
    }

    pub fn copy_perspective_origin_from(&mut self, other: &Self) {
        self.gecko.mPerspectiveOrigin[0].copy_from(&other.gecko.mPerspectiveOrigin[0]);
        self.gecko.mPerspectiveOrigin[1].copy_from(&other.gecko.mPerspectiveOrigin[1]);
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
</%self:impl_trait>

<%def name="simple_image_array_property(name, shorthand, field_name)">
    <%
        image_layers_field = "mImage" if shorthand == "background" else "mMask"
    %>
    pub fn copy_${shorthand}_${name}_from(&mut self, other: &Self) {
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        unsafe {
            Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field},
                                          other.gecko.${image_layers_field}.mLayers.len(),
                                          LayerType::${shorthand.title()});
        }
        for (layer, other) in self.gecko.${image_layers_field}.mLayers.iter_mut()
                                  .zip(other.gecko.${image_layers_field}.mLayers.iter())
                                  .take(other.gecko.${image_layers_field}
                                                   .${field_name}Count as usize) {
            layer.${field_name} = other.${field_name};
        }
        self.gecko.${image_layers_field}.${field_name}Count =
            other.gecko.${image_layers_field}.${field_name}Count;
    }

    pub fn set_${shorthand}_${name}(&mut self,
                                    v: longhands::${shorthand}_${name}::computed_value::T) {
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        unsafe {
          Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field}, v.0.len(),
                                        LayerType::${shorthand.title()});
        }

        self.gecko.${image_layers_field}.${field_name}Count = v.0.len() as u32;
        for (servo, geckolayer) in v.0.into_iter()
                                    .zip(self.gecko.${image_layers_field}.mLayers.iter_mut()) {
            geckolayer.${field_name} = {
                ${caller.body()}
            };
        }
    }
</%def>
<%def name="impl_common_image_layer_properties(shorthand)">
    <%
        image_layers_field = "mImage" if shorthand == "background" else "mMask"
    %>

    <%self:simple_image_array_property name="repeat" shorthand="${shorthand}" field_name="mRepeat">
        use properties::longhands::${shorthand}_repeat::single_value::computed_value::T;
        use gecko_bindings::structs::nsStyleImageLayers_Repeat;
        use gecko_bindings::structs::NS_STYLE_IMAGELAYER_REPEAT_REPEAT;
        use gecko_bindings::structs::NS_STYLE_IMAGELAYER_REPEAT_NO_REPEAT;
        use gecko_bindings::structs::NS_STYLE_IMAGELAYER_REPEAT_SPACE;
        use gecko_bindings::structs::NS_STYLE_IMAGELAYER_REPEAT_ROUND;

        let (repeat_x, repeat_y) = match servo {
          T::repeat_x => (NS_STYLE_IMAGELAYER_REPEAT_REPEAT,
                          NS_STYLE_IMAGELAYER_REPEAT_NO_REPEAT),
          T::repeat_y => (NS_STYLE_IMAGELAYER_REPEAT_NO_REPEAT,
                          NS_STYLE_IMAGELAYER_REPEAT_REPEAT),
          T::repeat  => (NS_STYLE_IMAGELAYER_REPEAT_REPEAT,
                         NS_STYLE_IMAGELAYER_REPEAT_REPEAT),
          T::space => (NS_STYLE_IMAGELAYER_REPEAT_SPACE,
                       NS_STYLE_IMAGELAYER_REPEAT_SPACE),
          T::round => (NS_STYLE_IMAGELAYER_REPEAT_ROUND,
                       NS_STYLE_IMAGELAYER_REPEAT_ROUND),
          T::no_repeat => (NS_STYLE_IMAGELAYER_REPEAT_NO_REPEAT,
                           NS_STYLE_IMAGELAYER_REPEAT_NO_REPEAT),
        };
        nsStyleImageLayers_Repeat {
              mXRepeat: repeat_x as u8,
              mYRepeat: repeat_y as u8,
        }
    </%self:simple_image_array_property>

    <%self:simple_image_array_property name="clip" shorthand="${shorthand}" field_name="mClip">
        use gecko_bindings::structs::StyleGeometryBox;
        use properties::longhands::${shorthand}_clip::single_value::computed_value::T;

        match servo {
            T::border_box => StyleGeometryBox::Border,
            T::padding_box => StyleGeometryBox::Padding,
            T::content_box => StyleGeometryBox::Content,
            % if shorthand == "mask":
            T::fill_box => StyleGeometryBox::Fill,
            T::stroke_box => StyleGeometryBox::Stroke,
            T::view_box => StyleGeometryBox::View,
            T::no_clip => StyleGeometryBox::NoClip,
            % elif shorthand == "background":
            T::text => StyleGeometryBox::Text,
            % endif
        }
    </%self:simple_image_array_property>

    <%self:simple_image_array_property name="origin" shorthand="${shorthand}" field_name="mOrigin">
        use gecko_bindings::structs::StyleGeometryBox;
        use properties::longhands::${shorthand}_origin::single_value::computed_value::T;

        match servo {
            T::border_box => StyleGeometryBox::Border,
            T::padding_box => StyleGeometryBox::Padding,
            T::content_box => StyleGeometryBox::Content,
            % if shorthand == "mask":
            T::fill_box => StyleGeometryBox::Fill,
            T::stroke_box => StyleGeometryBox::Stroke,
            T::view_box => StyleGeometryBox::View,
            % endif
        }
    </%self:simple_image_array_property>

    % for orientation in [("x", "Horizontal"), ("y", "Vertical")]:
    pub fn copy_${shorthand}_position_${orientation[0]}_from(&mut self, other: &Self) {
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        self.gecko.${image_layers_field}.mPosition${orientation[0].upper()}Count
            = cmp::min(1, other.gecko.${image_layers_field}.mPosition${orientation[0].upper()}Count);
        self.gecko.${image_layers_field}.mLayers.mFirstElement.mPosition =
            other.gecko.${image_layers_field}.mLayers.mFirstElement.mPosition;
        unsafe {
            Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field},
                                          other.gecko.${image_layers_field}.mLayers.len(),
                                          LayerType::${shorthand.capitalize()});
        }

        for (layer, other) in self.gecko.${image_layers_field}.mLayers.iter_mut()
                                  .zip(other.gecko.${image_layers_field}.mLayers.iter()) {
            layer.mPosition.m${orientation[0].upper()}Position
                = other.mPosition.m${orientation[0].upper()}Position;
        }
        self.gecko.${image_layers_field}.mPosition${orientation[0].upper()}Count
               = other.gecko.${image_layers_field}.mPosition${orientation[0].upper()}Count;
    }

    pub fn clone_${shorthand}_position_${orientation[0]}(&self)
        -> longhands::${shorthand}_position_${orientation[0]}::computed_value::T {
        use values::computed::position::${orientation[1]}Position;
        longhands::${shorthand}_position_${orientation[0]}::computed_value::T(
            self.gecko.${image_layers_field}.mLayers.iter()
                .take(self.gecko.${image_layers_field}.mPosition${orientation[0].upper()}Count as usize)
                .map(|position| ${orientation[1]}Position(position.mPosition.m${orientation[0].upper()}Position.into()))
                .collect()
        )
    }

    pub fn set_${shorthand}_position_${orientation[0]}(&mut self,
                                     v: longhands::${shorthand}_position_${orientation[0]}::computed_value::T) {
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        unsafe {
            Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field}, v.0.len(),
                                        LayerType::${shorthand.capitalize()});
        }

        self.gecko.${image_layers_field}.mPosition${orientation[0].upper()}Count = v.0.len() as u32;
        for (servo, geckolayer) in v.0.into_iter().zip(self.gecko.${image_layers_field}
                                                           .mLayers.iter_mut()) {
            geckolayer.mPosition.m${orientation[0].upper()}Position = servo.0.into();
        }
    }
    % endfor

    <%self:simple_image_array_property name="size" shorthand="${shorthand}" field_name="mSize">
        use gecko_bindings::structs::nsStyleImageLayers_Size_Dimension;
        use gecko_bindings::structs::nsStyleImageLayers_Size_DimensionType;
        use gecko_bindings::structs::{nsStyleCoord_CalcValue, nsStyleImageLayers_Size};
        use properties::longhands::background_size::single_value::computed_value::T;

        let mut width = nsStyleCoord_CalcValue::new();
        let mut height = nsStyleCoord_CalcValue::new();

        let (w_type, h_type) = match servo {
            T::Explicit(size) => {
                let mut w_type = nsStyleImageLayers_Size_DimensionType::eAuto;
                let mut h_type = nsStyleImageLayers_Size_DimensionType::eAuto;
                if let Some(w) = size.width.to_calc_value() {
                    width = w;
                    w_type = nsStyleImageLayers_Size_DimensionType::eLengthPercentage;
                }
                if let Some(h) = size.height.to_calc_value() {
                    height = h;
                    h_type = nsStyleImageLayers_Size_DimensionType::eLengthPercentage;
                }
                (w_type, h_type)
            }
            T::Cover => (nsStyleImageLayers_Size_DimensionType::eCover,
                         nsStyleImageLayers_Size_DimensionType::eCover),
            T::Contain => (nsStyleImageLayers_Size_DimensionType::eContain,
                         nsStyleImageLayers_Size_DimensionType::eContain),
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
        use properties::longhands::background_size::single_value::computed_value::{ExplicitSize, T};
        use values::computed::LengthOrPercentageOrAuto;

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
                    return T::Cover
                }
                if DimensionType::eContain as u8 == layer.mSize.mWidthType {
                    debug_assert!(layer.mSize.mHeightType == DimensionType::eContain as u8);
                    return T::Contain
                }

                T::Explicit(ExplicitSize {
                    width: to_servo(layer.mSize.mWidth._base, layer.mSize.mWidthType),
                    height: to_servo(layer.mSize.mHeight._base, layer.mSize.mHeightType),
                })
            }).collect()
        )
    }


    pub fn copy_${shorthand}_image_from(&mut self, other: &Self) {
        unsafe {
            Gecko_CopyImageValueFrom(&mut self.gecko.${image_layers_field}.mLayers.mFirstElement.mImage,
                                     &other.gecko.${image_layers_field}.mLayers.mFirstElement.mImage);
        }
    }

    #[allow(unused_variables)]
    pub fn set_${shorthand}_image(&mut self,
                                  images: longhands::${shorthand}_image::computed_value::T,
                                  cacheable: &mut bool) {
        use gecko_bindings::structs::nsStyleImageLayers_LayerType as LayerType;

        unsafe {
            // Prevent leaking of the last elements we did set
            for image in &mut self.gecko.${image_layers_field}.mLayers {
                Gecko_SetNullImageValue(&mut image.mImage)
            }
            // XXXManishearth clear mSourceURI for masks
            Gecko_EnsureImageLayersLength(&mut self.gecko.${image_layers_field}, images.0.len(),
                                          LayerType::${shorthand.title()});
        }

        self.gecko.${image_layers_field}.mImageCount = images.0.len() as u32;

        for (image, geckoimage) in images.0.into_iter().zip(self.gecko.${image_layers_field}
                                                                .mLayers.iter_mut()) {
            % if shorthand == "background":
                if let Some(image) = image.0 {
                    geckoimage.mImage.set(image, true, cacheable)
                }
            % else:
                use properties::longhands::mask_image::single_value::computed_value::T;
                match image {
                    T::Image(image) => geckoimage.mImage.set(image, false, cacheable),
                    _ => () // we need to support url valeus
                }
            % endif

        }
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

        // XXXManishearth Gecko does an optimization here where it only
        // fills things in if any of the properties have been set

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

    <%self:simple_image_array_property name="attachment" shorthand="background" field_name="mAttachment">
        use properties::longhands::background_attachment::single_value::computed_value::T;
        match servo {
            T::scroll => structs::NS_STYLE_IMAGELAYER_ATTACHMENT_SCROLL as u8,
            T::fixed => structs::NS_STYLE_IMAGELAYER_ATTACHMENT_FIXED as u8,
            T::local => structs::NS_STYLE_IMAGELAYER_ATTACHMENT_LOCAL as u8,
        }
    </%self:simple_image_array_property>

    <%self:simple_image_array_property name="blend_mode" shorthand="background" field_name="mBlendMode">
        use properties::longhands::background_blend_mode::single_value::computed_value::T;

        match servo {
            T::normal => structs::NS_STYLE_BLEND_NORMAL as u8,
            T::multiply => structs::NS_STYLE_BLEND_MULTIPLY as u8,
            T::screen => structs::NS_STYLE_BLEND_SCREEN as u8,
            T::overlay => structs::NS_STYLE_BLEND_OVERLAY as u8,
            T::darken => structs::NS_STYLE_BLEND_DARKEN as u8,
            T::lighten => structs::NS_STYLE_BLEND_LIGHTEN as u8,
            T::color_dodge => structs::NS_STYLE_BLEND_COLOR_DODGE as u8,
            T::color_burn => structs::NS_STYLE_BLEND_COLOR_BURN as u8,
            T::hard_light => structs::NS_STYLE_BLEND_HARD_LIGHT as u8,
            T::soft_light => structs::NS_STYLE_BLEND_SOFT_LIGHT as u8,
            T::difference => structs::NS_STYLE_BLEND_DIFFERENCE as u8,
            T::exclusion => structs::NS_STYLE_BLEND_EXCLUSION as u8,
            T::hue => structs::NS_STYLE_BLEND_HUE as u8,
            T::saturation => structs::NS_STYLE_BLEND_SATURATION as u8,
            T::color => structs::NS_STYLE_BLEND_COLOR as u8,
            T::luminosity => structs::NS_STYLE_BLEND_LUMINOSITY as u8,
        }
    </%self:simple_image_array_property>
</%self:impl_trait>

<%self:impl_trait style_struct_name="List"
                  skip_longhands="list-style-image list-style-type quotes -moz-image-region"
                  skip_additionals="*">

    pub fn set_list_style_image(&mut self, image: longhands::list_style_image::computed_value::T) {
        use values::Either;
        match image {
            Either::Second(_none) => {
                unsafe {
                    Gecko_SetListStyleImageNone(&mut self.gecko);
                }
            }
            Either::First(ref url) => {
                unsafe {
                    if let Some(ffi) = url.for_ffi() {
                        Gecko_SetListStyleImage(&mut self.gecko,
                                            ffi);
                    } else {
                        Gecko_SetListStyleImageNone(&mut self.gecko);
                    }
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

    pub fn set_list_style_type(&mut self, v: longhands::list_style_type::computed_value::T) {
        use properties::longhands::list_style_type::computed_value::T as Keyword;
        <%
            keyword = data.longhands_by_name["list-style-type"].keyword
            # The first four are @counter-styles
            # The rest have special fallback behavior
            special = """upper-roman lower-roman upper-alpha lower-alpha
                         japanese-informal japanese-formal korean-hangul-formal korean-hanja-informal
                         korean-hanja-formal simp-chinese-informal simp-chinese-formal
                         trad-chinese-informal trad-chinese-formal""".split()
        %>
        let result = match v {
            % for value in keyword.values_for('gecko'):
                % if value in special:
                    // Special keywords are implemented as @counter-styles
                    // and need to be manually set as strings
                    Keyword::${to_rust_ident(value)} => structs::${keyword.gecko_constant("none")},
                % else:
                    Keyword::${to_rust_ident(value)} =>
                        structs::${keyword.gecko_constant(value)},
                % endif
            % endfor
        };
        unsafe { Gecko_SetListStyleType(&mut self.gecko, result as u32); }
    }


    pub fn copy_list_style_type_from(&mut self, other: &Self) {
        unsafe {
            Gecko_CopyListStyleTypeFrom(&mut self.gecko, &other.gecko);
        }
    }

    pub fn set_quotes(&mut self, other: longhands::quotes::computed_value::T) {
        use gecko_bindings::bindings::Gecko_NewStyleQuoteValues;
        use gecko_bindings::sugar::refptr::UniqueRefPtr;
        use nsstring::nsCString;

        let mut refptr = unsafe {
            UniqueRefPtr::from_addrefed(Gecko_NewStyleQuoteValues(other.0.len() as u32))
        };

        for (servo, gecko) in other.0.into_iter().zip(refptr.mQuotePairs.iter_mut()) {
            gecko.first.assign_utf8(&nsCString::from(&*servo.0));
            gecko.second.assign_utf8(&nsCString::from(&*servo.1));
        }

        unsafe { self.gecko.mQuotes.set_move(refptr.get()) }
    }

    pub fn copy_quotes_from(&mut self, other: &Self) {
        unsafe { self.gecko.mQuotes.set(&other.gecko.mQuotes); }
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
                self.gecko.mImageRegion.height = rect.bottom.unwrap_or(Au(0)).0 - self.gecko.mImageRegion.y;
                self.gecko.mImageRegion.width = rect.right.unwrap_or(Au(0)).0 - self.gecko.mImageRegion.x;
            }
        }
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
    pub fn set_box_shadow(&mut self, v: longhands::box_shadow::computed_value::T) {

        self.gecko.mBoxShadow.replace_with_new(v.0.len() as u32);

        for (servo, gecko_shadow) in v.0.into_iter()
                                      .zip(self.gecko.mBoxShadow.iter_mut()) {

            gecko_shadow.mXOffset = servo.offset_x.0;
            gecko_shadow.mYOffset = servo.offset_y.0;
            gecko_shadow.mRadius = servo.blur_radius.0;
            gecko_shadow.mSpread = servo.spread_radius.0;
            gecko_shadow.mSpread = servo.spread_radius.0;
            gecko_shadow.mInset = servo.inset;
            gecko_shadow.mColor = match servo.color {
                Color::RGBA(rgba) => {
                    gecko_shadow.mHasColor = true;
                    convert_rgba_to_nscolor(&rgba)
                },
                // TODO handle currentColor
                // https://bugzilla.mozilla.org/show_bug.cgi?id=760345
                Color::CurrentColor => 0,
            }

        }
    }

    pub fn copy_box_shadow_from(&mut self, other: &Self) {
        self.gecko.mBoxShadow.copy_from(&other.gecko.mBoxShadow);
    }

    pub fn clone_box_shadow(&self) -> longhands::box_shadow::computed_value::T {
        let buf = self.gecko.mBoxShadow.iter().map(|shadow| {
            longhands::box_shadow::single_value::computed_value::T {
                offset_x: Au(shadow.mXOffset),
                offset_y: Au(shadow.mYOffset),
                blur_radius: Au(shadow.mRadius),
                spread_radius: Au(shadow.mSpread),
                inset: shadow.mInset,
                color: Color::RGBA(convert_nscolor_to_rgba(shadow.mColor)),
            }
        }).collect();
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

    pub fn set_filter(&mut self, v: longhands::filter::computed_value::T) {
        use properties::longhands::filter::computed_value::Filter::*;
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

        unsafe {
            Gecko_ResetFilters(&mut self.gecko, v.filters.len());
        }
        debug_assert!(v.filters.len() == self.gecko.mFilters.len());

        for (servo, gecko_filter) in v.filters.into_iter().zip(self.gecko.mFilters.iter_mut()) {
            //TODO: URL, drop-shadow
            match servo {
                Blur(len)          => fill_filter(NS_STYLE_FILTER_BLUR,
                                                  CoordDataValue::Coord(len.0),
                                                  gecko_filter),
                Brightness(factor) => fill_filter(NS_STYLE_FILTER_BRIGHTNESS,
                                                  CoordDataValue::Factor(factor),
                                                  gecko_filter),
                Contrast(factor)   => fill_filter(NS_STYLE_FILTER_CONTRAST,
                                                  CoordDataValue::Factor(factor),
                                                  gecko_filter),
                Grayscale(factor)  => fill_filter(NS_STYLE_FILTER_GRAYSCALE,
                                                  CoordDataValue::Factor(factor),
                                                  gecko_filter),
                HueRotate(angle)   => fill_filter(NS_STYLE_FILTER_HUE_ROTATE,
                                                  CoordDataValue::Radian(angle.radians()),
                                                  gecko_filter),
                Invert(factor)     => fill_filter(NS_STYLE_FILTER_INVERT,
                                                  CoordDataValue::Factor(factor),
                                                  gecko_filter),
                Opacity(factor)    => fill_filter(NS_STYLE_FILTER_OPACITY,
                                                  CoordDataValue::Factor(factor),
                                                  gecko_filter),
                Saturate(factor)   => fill_filter(NS_STYLE_FILTER_SATURATE,
                                                  CoordDataValue::Factor(factor),
                                                  gecko_filter),
                Sepia(factor)      => fill_filter(NS_STYLE_FILTER_SEPIA,
                                                  CoordDataValue::Factor(factor),
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
                    gecko_shadow.mArray[0].mXOffset = shadow.offset_x.0;
                    gecko_shadow.mArray[0].mYOffset = shadow.offset_y.0;
                    gecko_shadow.mArray[0].mRadius = shadow.blur_radius.0;
                    // mSpread is not supported in the spec, so we leave it as 0
                    gecko_shadow.mArray[0].mInset = false; // Not supported in spec level 1

                    gecko_shadow.mArray[0].mColor = match shadow.color {
                        Color::RGBA(rgba) => {
                            gecko_shadow.mArray[0].mHasColor = true;
                            convert_rgba_to_nscolor(&rgba)
                        },
                        // TODO handle currentColor
                        // https://bugzilla.mozilla.org/show_bug.cgi?id=760345
                        Color::CurrentColor => 0,
                    };
                }
                Url(ref url) => {
                    unsafe {
                        if let Some(ffi) = url.for_ffi() {
                            bindings::Gecko_nsStyleFilter_SetURLValue(gecko_filter, ffi);
                        }
                    }
                }
            }
        }
    }

    pub fn copy_filter_from(&mut self, other: &Self) {
        unsafe {
            Gecko_CopyFiltersFrom(&other.gecko as *const _ as *mut _, &mut self.gecko);
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

</%self:impl_trait>


<%self:impl_trait style_struct_name="InheritedText"
                  skip_longhands="text-align text-emphasis-style text-shadow line-height letter-spacing word-spacing
                                  -webkit-text-stroke-width text-emphasis-position -moz-tab-size">

    <% text_align_keyword = Keyword("text-align", "start end left right center justify -moz-center -moz-left " +
                                                  "-moz-right match-parent char") %>
    ${impl_keyword('text_align', 'mTextAlign', text_align_keyword, need_clone=False)}

    pub fn set_text_shadow(&mut self, v: longhands::text_shadow::computed_value::T) {
        self.gecko.mTextShadow.replace_with_new(v.0.len() as u32);

        for (servo, gecko_shadow) in v.0.into_iter()
                                      .zip(self.gecko.mTextShadow.iter_mut()) {

            gecko_shadow.mXOffset = servo.offset_x.0;
            gecko_shadow.mYOffset = servo.offset_y.0;
            gecko_shadow.mRadius = servo.blur_radius.0;
            gecko_shadow.mHasColor = false;
            gecko_shadow.mColor = match servo.color {
                Color::RGBA(rgba) => {
                    gecko_shadow.mHasColor = true;
                    convert_rgba_to_nscolor(&rgba)
                },
                // TODO handle currentColor
                // https://bugzilla.mozilla.org/show_bug.cgi?id=760345
                Color::CurrentColor => 0,
            }

        }
    }

    pub fn copy_text_shadow_from(&mut self, other: &Self) {
        self.gecko.mTextShadow.copy_from(&other.gecko.mTextShadow);
    }

    pub fn clone_text_shadow(&self) -> longhands::text_shadow::computed_value::T {

        let buf = self.gecko.mTextShadow.iter().map(|shadow| {
            longhands::text_shadow::computed_value::TextShadow {
                offset_x: Au(shadow.mXOffset),
                offset_y: Au(shadow.mYOffset),
                blur_radius: Au(shadow.mRadius),
                color: Color::RGBA(convert_nscolor_to_rgba(shadow.mColor)),
            }

        }).collect();
        longhands::text_shadow::computed_value::T(buf)
    }

    pub fn set_line_height(&mut self, v: longhands::line_height::computed_value::T) {
        use properties::longhands::line_height::computed_value::T;
        // FIXME: Align binary representations and ditch |match| for cast + static_asserts
        let en = match v {
            T::Normal => CoordDataValue::Normal,
            T::Length(val) => CoordDataValue::Coord(val.0),
            T::Number(val) => CoordDataValue::Factor(val),
            T::MozBlockHeight =>
                    CoordDataValue::Enumerated(structs::NS_STYLE_LINE_HEIGHT_BLOCK_HEIGHT),
        };
        self.gecko.mLineHeight.set_value(en);
    }

    pub fn clone_line_height(&self) -> longhands::line_height::computed_value::T {
        use properties::longhands::line_height::computed_value::T;
        return match self.gecko.mLineHeight.as_value() {
            CoordDataValue::Normal => T::Normal,
            CoordDataValue::Coord(coord) => T::Length(Au(coord)),
            CoordDataValue::Factor(n) => T::Number(n),
            CoordDataValue::Enumerated(val) if val == structs::NS_STYLE_LINE_HEIGHT_BLOCK_HEIGHT =>
                T::MozBlockHeight,
            _ => {
                debug_assert!(false);
                T::MozBlockHeight
            }
        }
    }

    <%call expr="impl_coord_copy('line_height', 'mLineHeight')"></%call>

    pub fn set_letter_spacing(&mut self, v: longhands::letter_spacing::computed_value::T) {
        match v.0 {
            Some(au) => self.gecko.mLetterSpacing.set(au),
            None => self.gecko.mLetterSpacing.set_value(CoordDataValue::Normal)
        }
    }

    <%call expr="impl_coord_copy('letter_spacing', 'mLetterSpacing')"></%call>

    pub fn set_word_spacing(&mut self, v: longhands::word_spacing::computed_value::T) {
        match v.0 {
            Some(lop) => self.gecko.mWordSpacing.set(lop),
            // https://drafts.csswg.org/css-text-3/#valdef-word-spacing-normal
            None => self.gecko.mWordSpacing.set_value(CoordDataValue::Coord(0)),
        }
    }

    <%call expr="impl_coord_copy('word_spacing', 'mWordSpacing')"></%call>

    fn clear_text_emphasis_style_if_string(&mut self) {
        use nsstring::nsString;
        if self.gecko.mTextEmphasisStyle == structs::NS_STYLE_TEXT_EMPHASIS_STYLE_STRING as u8 {
            self.gecko.mTextEmphasisStyleString.assign(&nsString::new());
            self.gecko.mTextEmphasisStyle = structs::NS_STYLE_TEXT_EMPHASIS_STYLE_NONE as u8;
        }
    }

    pub fn set_text_emphasis_position(&mut self, v: longhands::text_emphasis_position::computed_value::T) {
        use properties::longhands::text_emphasis_position::*;

        let mut result = match v.0 {
            HorizontalWritingModeValue::Over => structs::NS_STYLE_TEXT_EMPHASIS_POSITION_OVER as u8,
            HorizontalWritingModeValue::Under => structs::NS_STYLE_TEXT_EMPHASIS_POSITION_UNDER as u8,
        };
        match v.1 {
            VerticalWritingModeValue::Right => {
                result |= structs::NS_STYLE_TEXT_EMPHASIS_POSITION_RIGHT as u8;
            }
            VerticalWritingModeValue::Left => {
                result |= structs::NS_STYLE_TEXT_EMPHASIS_POSITION_LEFT as u8;
            }
        }
        self.gecko.mTextEmphasisPosition = result;
    }

    <%call expr="impl_simple_copy('text_emphasis_position', 'mTextEmphasisPosition')"></%call>

    pub fn set_text_emphasis_style(&mut self, v: longhands::text_emphasis_style::computed_value::T) {
        use nsstring::nsCString;
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
        self.gecko.mTextEmphasisStyleString.assign_utf8(&nsCString::from(s));
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

    <%call expr="impl_app_units('_webkit_text_stroke_width', 'mWebkitTextStrokeWidth', need_clone=False)"></%call>

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

    <%call expr="impl_coord_copy('_moz_tab_size', 'mTabSize')"></%call>

</%self:impl_trait>

<%self:impl_trait style_struct_name="Text"
                  skip_longhands="text-decoration-line text-overflow"
                  skip_additionals="*">

    pub fn set_text_decoration_line(&mut self, v: longhands::text_decoration_line::computed_value::T) {
        let mut bits: u8 = 0;
        if v.contains(longhands::text_decoration_line::UNDERLINE) {
            bits |= structs::NS_STYLE_TEXT_DECORATION_LINE_UNDERLINE as u8;
        }
        if v.contains(longhands::text_decoration_line::OVERLINE) {
            bits |= structs::NS_STYLE_TEXT_DECORATION_LINE_OVERLINE as u8;
        }
        if v.contains(longhands::text_decoration_line::LINE_THROUGH) {
            bits |= structs::NS_STYLE_TEXT_DECORATION_LINE_LINE_THROUGH as u8;
        }
        if v.contains(longhands::text_decoration_line::BLINK) {
            bits |= structs::NS_STYLE_TEXT_DECORATION_LINE_BLINK as u8;
        }
        if v.contains(longhands::text_decoration_line::COLOR_OVERRIDE) {
            bits |= structs::NS_STYLE_TEXT_DECORATION_LINE_OVERRIDE_ALL as u8;
        }
        self.gecko.mTextDecorationLine = bits;
    }

    ${impl_simple_copy('text_decoration_line', 'mTextDecorationLine')}


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
        use properties::longhands::text_overflow::{SpecifiedValue, Side};

        fn set(side: &mut nsStyleTextOverflowSide, value: &Side) {
            use nsstring::nsCString;
            let ty = match *value {
                Side::Clip => structs::NS_STYLE_TEXT_OVERFLOW_CLIP,
                Side::Ellipsis => structs::NS_STYLE_TEXT_OVERFLOW_ELLIPSIS,
                Side::String(ref s) => {
                    side.mString.assign_utf8(&nsCString::from(&**s));
                    structs::NS_STYLE_TEXT_OVERFLOW_STRING
                }
            };
            side.mType = ty as u8;
        }

        self.clear_overflow_sides_if_string();
        if v.second.is_none() {
            self.gecko.mTextOverflow.mLogicalDirections = true;
        }

        let SpecifiedValue { ref first, ref second } = v;
        let second = second.as_ref().unwrap_or(&first);

        set(&mut self.gecko.mTextOverflow.mLeft, first);
        set(&mut self.gecko.mTextOverflow.mRight, second);
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

<% skip_svg_longhands = """
mask-mode mask-repeat mask-clip mask-origin mask-composite mask-position-x mask-position-y mask-size mask-image
clip-path
"""
%>
<%self:impl_trait style_struct_name="SVG"
                  skip_longhands="${skip_svg_longhands}"
                  skip_additionals="*">

    <% impl_common_image_layer_properties("mask") %>

    <%self:simple_image_array_property name="mode" shorthand="mask" field_name="mMaskMode">
        use properties::longhands::mask_mode::single_value::computed_value::T;

        match servo {
          T::alpha => structs::NS_STYLE_MASK_MODE_ALPHA as u8,
          T::luminance => structs::NS_STYLE_MASK_MODE_LUMINANCE as u8,
          T::match_source => structs::NS_STYLE_MASK_MODE_MATCH_SOURCE as u8,
        }
    </%self:simple_image_array_property>
    <%self:simple_image_array_property name="composite" shorthand="mask" field_name="mComposite">
        use properties::longhands::mask_composite::single_value::computed_value::T;

        match servo {
            T::add => structs::NS_STYLE_MASK_COMPOSITE_ADD as u8,
            T::subtract => structs::NS_STYLE_MASK_COMPOSITE_SUBTRACT as u8,
            T::intersect => structs::NS_STYLE_MASK_COMPOSITE_INTERSECT as u8,
            T::exclude => structs::NS_STYLE_MASK_COMPOSITE_EXCLUDE as u8,
        }
    </%self:simple_image_array_property>
    pub fn set_clip_path(&mut self, v: longhands::clip_path::computed_value::T) {
        use gecko_bindings::bindings::{Gecko_NewBasicShape, Gecko_DestroyClipPath};
        use gecko_bindings::structs::StyleGeometryBox;
        use gecko_bindings::structs::{StyleBasicShape, StyleBasicShapeType, StyleShapeSourceType};
        use gecko_bindings::structs::{StyleFillRule, StyleShapeSource};
        use gecko::conversions::basic_shape::set_corners_from_radius;
        use gecko::values::GeckoStyleCoordConvertible;
        use values::computed::basic_shape::*;
        let ref mut clip_path = self.gecko.mClipPath;
        // clean up existing struct
        unsafe { Gecko_DestroyClipPath(clip_path) };

        clip_path.mType = StyleShapeSourceType::None;

        match v {
            ShapeSource::Url(ref url) => {
                unsafe {
                    if let Some(ffi) = url.for_ffi() {
                       bindings::Gecko_StyleClipPath_SetURLValue(clip_path, ffi);
                    }
                }
            }
            ShapeSource::None => {} // don't change the type
            ShapeSource::Box(reference) => {
                clip_path.mReferenceBox = reference.into();
                clip_path.mType = StyleShapeSourceType::Box;
            }
            ShapeSource::Shape(servo_shape, maybe_box) => {
                clip_path.mReferenceBox = maybe_box.map(Into::into)
                                                   .unwrap_or(StyleGeometryBox::NoBox);
                clip_path.mType = StyleShapeSourceType::Shape;

                fn init_shape(clip_path: &mut StyleShapeSource, ty: StyleBasicShapeType) -> &mut StyleBasicShape {
                    unsafe {
                        // We have to be very careful to avoid a copy here!
                        let ref mut union = clip_path.__bindgen_anon_1;
                        let mut shape: &mut *mut StyleBasicShape = union.mBasicShape.as_mut();
                        *shape = Gecko_NewBasicShape(ty);
                        &mut **shape
                    }
                }
                match servo_shape {
                    BasicShape::Inset(rect) => {
                        let mut shape = init_shape(clip_path, StyleBasicShapeType::Inset);
                        unsafe { shape.mCoordinates.set_len(4) };

                        // set_len() can't call constructors, so the coordinates
                        // can contain any value. set_value() attempts to free
                        // allocated coordinates, so we don't want to feed it
                        // garbage values which it may misinterpret.
                        // Instead, we use leaky_set_value to blindly overwrite
                        // the garbage data without
                        // attempting to clean up.
                        shape.mCoordinates[0].leaky_set_null();
                        rect.top.to_gecko_style_coord(&mut shape.mCoordinates[0]);
                        shape.mCoordinates[1].leaky_set_null();
                        rect.right.to_gecko_style_coord(&mut shape.mCoordinates[1]);
                        shape.mCoordinates[2].leaky_set_null();
                        rect.bottom.to_gecko_style_coord(&mut shape.mCoordinates[2]);
                        shape.mCoordinates[3].leaky_set_null();
                        rect.left.to_gecko_style_coord(&mut shape.mCoordinates[3]);

                        set_corners_from_radius(rect.round, &mut shape.mRadius);
                    }
                    BasicShape::Circle(circ) => {
                        let mut shape = init_shape(clip_path, StyleBasicShapeType::Circle);
                        unsafe { shape.mCoordinates.set_len(1) };
                        shape.mCoordinates[0].leaky_set_null();
                        circ.radius.to_gecko_style_coord(&mut shape.mCoordinates[0]);

                        shape.mPosition = circ.position.into();
                    }
                    BasicShape::Ellipse(el) => {
                        let mut shape = init_shape(clip_path, StyleBasicShapeType::Ellipse);
                        unsafe { shape.mCoordinates.set_len(2) };
                        shape.mCoordinates[0].leaky_set_null();
                        el.semiaxis_x.to_gecko_style_coord(&mut shape.mCoordinates[0]);
                        shape.mCoordinates[1].leaky_set_null();
                        el.semiaxis_y.to_gecko_style_coord(&mut shape.mCoordinates[1]);

                        shape.mPosition = el.position.into();
                    }
                    BasicShape::Polygon(poly) => {
                        let mut shape = init_shape(clip_path, StyleBasicShapeType::Polygon);
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

    pub fn copy_clip_path_from(&mut self, other: &Self) {
        use gecko_bindings::bindings::Gecko_CopyClipPathValueFrom;
        unsafe {
            Gecko_CopyClipPathValueFrom(&mut self.gecko.mClipPath, &other.gecko.mClipPath);
        }
    }
</%self:impl_trait>

<%self:impl_trait style_struct_name="InheritedSVG"
                  skip_longhands="paint-order stroke-dasharray"
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

    pub fn set_stroke_dasharray(&mut self, v: longhands::stroke_dasharray::computed_value::T) {
        unsafe {
            bindings::Gecko_nsStyleSVG_SetDashArrayLength(&mut self.gecko, v.0.len() as u32);
        }

        for (mut gecko, servo) in self.gecko.mStrokeDasharray.iter_mut().zip(v.0.into_iter()) {
            match servo {
                Either::First(lop) => gecko.set(lop),
                Either::Second(number) => gecko.set_value(CoordDataValue::Factor(number)),
            }
        }
    }

    pub fn copy_stroke_dasharray_from(&mut self, other: &Self) {
        unsafe {
            bindings::Gecko_nsStyleSVG_CopyDashArray(&mut self.gecko, &other.gecko);
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
                  skip_longhands="cursor">
    pub fn set_cursor(&mut self, v: longhands::cursor::computed_value::T) {
        use properties::longhands::cursor::computed_value::Keyword;
        use style_traits::cursor::Cursor;

        self.gecko.mCursor = match v.keyword {
            Keyword::AutoCursor => structs::NS_STYLE_CURSOR_AUTO,
            Keyword::SpecifiedCursor(cursor) => match cursor {
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
            }
        } as u8;

        unsafe {
            Gecko_SetCursorArrayLength(&mut self.gecko, v.images.len());
        }
        for i in 0..v.images.len() {
            let image = &v.images[i];
            let extra_data = image.url.extra_data();
            let (ptr, len) = match image.url.as_slice_components() {
                Ok(value) | Err(value) => value,
            };
            unsafe {
                Gecko_SetCursorImage(&mut self.gecko.mCursorImages[i],
                                     ptr, len as u32,
                                     extra_data.base.get(),
                                     extra_data.referrer.get(),
                                     extra_data.principal.get());
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
</%self:impl_trait>

<%self:impl_trait style_struct_name="Column"
                  skip_longhands="column-count column-rule-width">

    #[allow(unused_unsafe)]
    pub fn set_column_count(&mut self, v: longhands::column_count::computed_value::T) {
        use gecko_bindings::structs::{NS_STYLE_COLUMN_COUNT_AUTO, nsStyleColumn_kMaxColumnCount};

        self.gecko.mColumnCount = match v.0 {
            Some(number) => unsafe {
                cmp::min(number, nsStyleColumn_kMaxColumnCount)
            },
            None => NS_STYLE_COLUMN_COUNT_AUTO
        };
    }

    ${impl_simple_copy('column_count', 'mColumnCount')}

    <% impl_app_units("column_rule_width", "mColumnRuleWidth", need_clone=True,
                      round_to_pixels=True) %>
</%self:impl_trait>

<%self:impl_trait style_struct_name="Counters"
                  skip_longhands="content">
    pub fn set_content(&mut self, v: longhands::content::computed_value::T) {
        use properties::longhands::content::computed_value::T;
        use properties::longhands::content::computed_value::ContentItem;
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

        match v {
            T::none |
            T::normal => {
                // Ensure destructors run, otherwise we could leak.
                if !self.gecko.mContents.is_empty() {
                    unsafe {
                        Gecko_ClearAndResizeStyleContents(&mut self.gecko, 0);
                    }
                }
            },
            T::Content(items) => {
                unsafe {
                    Gecko_ClearAndResizeStyleContents(&mut self.gecko,
                                                      items.len() as u32);
                }
                for (i, item) in items.into_iter().enumerate() {
                    // TODO: Servo lacks support for attr(), and URIs.
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
                        ContentItem::OpenQuote
                            => self.gecko.mContents[i].mType = eStyleContentType_OpenQuote,
                        ContentItem::CloseQuote
                            => self.gecko.mContents[i].mType = eStyleContentType_CloseQuote,
                        ContentItem::NoOpenQuote
                            => self.gecko.mContents[i].mType = eStyleContentType_NoOpenQuote,
                        ContentItem::NoCloseQuote
                            => self.gecko.mContents[i].mType = eStyleContentType_NoCloseQuote,
                        ContentItem::MozAltContent
                            => self.gecko.mContents[i].mType = eStyleContentType_AltContent,
                        ContentItem::Counter(..) |
                        ContentItem::Counters(..)
                            => self.gecko.mContents[i].mType = eStyleContentType_Uninitialized,
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
</%self:impl_trait>

<%self:impl_trait style_struct_name="XUL"
                  skip_longhands="-moz-stack-sizing">

    #[allow(non_snake_case)]
    pub fn set__moz_stack_sizing(&mut self, v: longhands::_moz_stack_sizing::computed_value::T) {
        use properties::longhands::_moz_stack_sizing::computed_value::T;
        self.gecko.mStretchStack = v == T::stretch_to_fit;
    }

    ${impl_simple_copy('_moz_stack_sizing', 'mStretchStack')}
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
