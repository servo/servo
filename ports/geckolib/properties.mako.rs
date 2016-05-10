/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// `data` comes from components/style/properties.mako.rs; see build.rs for more details.

<%!
    from data import to_rust_ident
    from data import Keyword
%>

use app_units::Au;
% for style_struct in data.style_structs:
use gecko_style_structs::${style_struct.gecko_ffi_name};
use bindings::Gecko_Construct_${style_struct.gecko_ffi_name};
use bindings::Gecko_CopyConstruct_${style_struct.gecko_ffi_name};
use bindings::Gecko_Destroy_${style_struct.gecko_ffi_name};
% endfor
use bindings::{Gecko_CopyListStyleTypeFrom, Gecko_SetListStyleType};
use gecko_style_structs;
use glue::ArcHelpers;
use heapsize::HeapSizeOf;
use std::fmt::{self, Debug};
use std::mem::{transmute, zeroed};
use std::sync::Arc;
use style::custom_properties::ComputedValuesMap;
use style::logical_geometry::WritingMode;
use style::properties::{CascadePropertyFn, ServoComputedValues, ComputedValues};
use style::properties::longhands;
use style::properties::make_cascade_vec;
use style::properties::style_struct_traits::*;
use gecko_style_structs::{nsStyleUnion, nsStyleUnit};
use values::{ToGeckoStyleCoord, convert_rgba_to_nscolor, convert_nscolor_to_rgba};
use values::round_border_to_device_pixels;

#[derive(Clone)]
pub struct GeckoComputedValues {
    % for style_struct in data.style_structs:
    ${style_struct.ident}: Arc<${style_struct.gecko_struct_name}>,
    % endfor

    custom_properties: Option<Arc<ComputedValuesMap>>,
    shareable: bool,
    pub writing_mode: WritingMode,
    pub root_font_size: Au,
}

impl GeckoComputedValues {
    pub fn inherit_from(parent: &Arc<Self>) -> Arc<Self> {
        Arc::new(GeckoComputedValues {
            custom_properties: parent.custom_properties.clone(),
            shareable: parent.shareable,
            writing_mode: parent.writing_mode,
            root_font_size: parent.root_font_size,
            % for style_struct in data.style_structs:
            % if style_struct.inherited:
            ${style_struct.ident}: parent.${style_struct.ident}.clone(),
            % else:
            ${style_struct.ident}: Self::initial_values().${style_struct.ident}.clone(),
            % endif
            % endfor
        })
    }
}

impl ComputedValues for GeckoComputedValues {
% for style_struct in data.style_structs:
    type Concrete${style_struct.trait_name} = ${style_struct.gecko_struct_name};
% endfor

    // These will go away, and we will never implement them.
    fn as_servo<'a>(&'a self) -> &'a ServoComputedValues { unimplemented!() }
    fn as_servo_mut<'a>(&'a mut self) -> &'a mut ServoComputedValues { unimplemented!() }

    fn new(custom_properties: Option<Arc<ComputedValuesMap>>,
           shareable: bool,
           writing_mode: WritingMode,
           root_font_size: Au,
            % for style_struct in data.style_structs:
           ${style_struct.ident}: Arc<${style_struct.gecko_struct_name}>,
            % endfor
    ) -> Self {
        GeckoComputedValues {
            custom_properties: custom_properties,
            shareable: shareable,
            writing_mode: writing_mode,
            root_font_size: root_font_size,
            % for style_struct in data.style_structs:
            ${style_struct.ident}: ${style_struct.ident},
            % endfor
        }
    }

    fn style_for_child_text_node(parent: &Arc<Self>) -> Arc<Self> {
        // Gecko expects text nodes to be styled as if they were elements that
        // matched no rules (that is, inherited style structs are inherited and
        // non-inherited style structs are set to their initial values).
        GeckoComputedValues::inherit_from(parent)
    }

    fn initial_values() -> &'static Self { &*INITIAL_GECKO_VALUES }

    fn do_cascade_property<F: FnOnce(&Vec<Option<CascadePropertyFn<Self>>>)>(f: F) {
        CASCADE_PROPERTY.with(|x| f(x));
    }

    % for style_struct in data.style_structs:
    #[inline]
    fn clone_${style_struct.trait_name_lower}(&self) -> Arc<Self::Concrete${style_struct.trait_name}> {
        self.${style_struct.ident}.clone()
    }
    #[inline]
    fn get_${style_struct.trait_name_lower}<'a>(&'a self) -> &'a Self::Concrete${style_struct.trait_name} {
        &self.${style_struct.ident}
    }
    #[inline]
    fn mutate_${style_struct.trait_name_lower}<'a>(&'a mut self) -> &'a mut Self::Concrete${style_struct.trait_name} {
        Arc::make_mut(&mut self.${style_struct.ident})
    }
    % endfor

    fn custom_properties(&self) -> Option<Arc<ComputedValuesMap>> { self.custom_properties.as_ref().map(|x| x.clone())}
    fn root_font_size(&self) -> Au { self.root_font_size }
    fn set_root_font_size(&mut self, s: Au) { self.root_font_size = s; }
    fn set_writing_mode(&mut self, mode: WritingMode) { self.writing_mode = mode; }

    // FIXME(bholley): Implement this properly.
    #[inline]
    fn is_multicol(&self) -> bool { false }
}

<%def name="declare_style_struct(style_struct)">
#[derive(Clone, HeapSizeOf, Debug)]
pub struct ${style_struct.gecko_struct_name} {
    gecko: ${style_struct.gecko_ffi_name},
}
</%def>

<%def name="impl_simple_setter(ident, gecko_ffi_name)">
    fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        ${set_gecko_property(gecko_ffi_name, "v")}
    }
</%def>

<%def name="impl_simple_copy(ident, gecko_ffi_name)">
    fn copy_${ident}_from(&mut self, other: &Self) {
        self.gecko.${gecko_ffi_name} = other.gecko.${gecko_ffi_name};
    }
</%def>

<%!
def is_border_style_masked(ffi_name):
    return ffi_name.split("[")[0] in ["mBorderStyle", "mOutlineStyle", "mTextDecorationStyle"]

def get_gecko_property(ffi_name):
    if is_border_style_masked(ffi_name):
        return "(self.gecko.%s & (gecko_style_structs::BORDER_STYLE_MASK as u8))" % ffi_name
    return "self.gecko.%s" % ffi_name

def set_gecko_property(ffi_name, expr):
    if is_border_style_masked(ffi_name):
        return "self.gecko.%s &= !(gecko_style_structs::BORDER_STYLE_MASK as u8);" % ffi_name + \
               "self.gecko.%s |= %s as u8;" % (ffi_name, expr)
    elif ffi_name == "__LIST_STYLE_TYPE__":
        return "unsafe { Gecko_SetListStyleType(&mut self.gecko, %s as u32); }" % expr
    return "self.gecko.%s = %s;" % (ffi_name, expr)
%>

<%def name="impl_keyword_setter(ident, gecko_ffi_name, keyword)">
    fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        use gecko_style_structs as gss;
        use style::properties::longhands::${ident}::computed_value::T as Keyword;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        let result = match v {
            % for value in keyword.values_for('gecko'):
                Keyword::${to_rust_ident(value)} => gss::${keyword.gecko_constant(value)} as u8,
            % endfor
        };
        ${set_gecko_property(gecko_ffi_name, "result")}
    }
</%def>

<%def name="impl_keyword_clone(ident, gecko_ffi_name, keyword)">
    fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        use gecko_style_structs as gss;
        use style::properties::longhands::${ident}::computed_value::T as Keyword;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        match ${get_gecko_property(gecko_ffi_name)} as u32 {
            % for value in keyword.values_for('gecko'):
            gss::${keyword.gecko_constant(value)} => Keyword::${to_rust_ident(value)},
            % endfor
            x => panic!("Found unexpected value in style struct for ${ident} property: {}", x),
        }
    }
</%def>

<%def name="clear_color_flags(color_flags_ffi_name)">
    % if color_flags_ffi_name:
    self.gecko.${color_flags_ffi_name} &= !(gecko_style_structs::BORDER_COLOR_SPECIAL as u8);
    % endif
</%def>

<%def name="set_current_color_flag(color_flags_ffi_name)">
    % if color_flags_ffi_name:
    self.gecko.${color_flags_ffi_name} |= gecko_style_structs::BORDER_COLOR_FOREGROUND as u8;
    % else:
    // FIXME(heycam): This is a Gecko property that doesn't store currentColor
    // as a computed value.  These are currently handled by converting
    // currentColor to the current value of the color property at computed
    // value time, but we don't have access to the Color struct here.
    // In the longer term, Gecko should store currentColor as a computed
    // value, so that we don't need to do this:
    // https://bugzilla.mozilla.org/show_bug.cgi?id=760345
    unimplemented!();
    % endif
</%def>

<%def name="get_current_color_flag_from(field)">
    (${field} & (gecko_style_structs::BORDER_COLOR_FOREGROUND as u8)) != 0
</%def>

<%def name="impl_color_setter(ident, gecko_ffi_name, color_flags_ffi_name=None)">
    #[allow(unreachable_code)]
    fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        use cssparser::Color;
        ${clear_color_flags(color_flags_ffi_name)}
        let result = match v {
            Color::CurrentColor => {
                ${set_current_color_flag(color_flags_ffi_name)}
                0
            },
            Color::RGBA(rgba) => convert_rgba_to_nscolor(&rgba),
        };
        ${set_gecko_property(gecko_ffi_name, "result")}
    }
</%def>

<%def name="impl_color_copy(ident, gecko_ffi_name, color_flags_ffi_name=None)">
    fn copy_${ident}_from(&mut self, other: &Self) {
        % if color_flags_ffi_name:
        ${clear_color_flags(color_flags_ffi_name)}
        if ${get_current_color_flag_from("other.gecko." + color_flags_ffi_name)} {
            ${set_current_color_flag(color_flags_ffi_name)}
        }
        % endif
        self.gecko.${gecko_ffi_name} = other.gecko.${gecko_ffi_name};
    }
</%def>

<%def name="impl_keyword(ident, gecko_ffi_name, keyword, need_clone)">
<%call expr="impl_keyword_setter(ident, gecko_ffi_name, keyword)"></%call>
<%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
%if need_clone:
<%call expr="impl_keyword_clone(ident, gecko_ffi_name, keyword)"></%call>
% endif
</%def>

<%def name="impl_simple(ident, gecko_ffi_name)">
<%call expr="impl_simple_setter(ident, gecko_ffi_name)"></%call>
<%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
</%def>

<%def name="impl_color(ident, gecko_ffi_name, color_flags_ffi_name=None)">
<%call expr="impl_color_setter(ident, gecko_ffi_name, color_flags_ffi_name)"></%call>
<%call expr="impl_color_copy(ident, gecko_ffi_name, color_flags_ffi_name)"></%call>
</%def>

<%def name="impl_app_units(ident, gecko_ffi_name, need_clone, round_to_pixels=False)">
    fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        % if round_to_pixels:
        let au_per_device_px = Au(self.gecko.mTwipsPerPixel);
        self.gecko.${gecko_ffi_name} = round_border_to_device_pixels(v, au_per_device_px).0;
        % else:
        self.gecko.${gecko_ffi_name} = v.0;
        % endif
    }
<%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
%if need_clone:
    fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        Au(self.gecko.${gecko_ffi_name})
    }
% endif
</%def>

<%def name="impl_split_style_coord(ident, unit_ffi_name, union_ffi_name)">
    fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        v.to_gecko_style_coord(&mut self.gecko.${unit_ffi_name},
                               &mut self.gecko.${union_ffi_name});
    }
    fn copy_${ident}_from(&mut self, other: &Self) {
        use gecko_style_structs::nsStyleUnit::eStyleUnit_Calc;
        debug_assert!(self.gecko.${unit_ffi_name} != eStyleUnit_Calc,
                      "stylo: Can't yet handle refcounted Calc");
        self.gecko.${unit_ffi_name} =  other.gecko.${unit_ffi_name};
        self.gecko.${union_ffi_name} = other.gecko.${union_ffi_name};
    }
</%def>

<%def name="impl_style_coord(ident, gecko_ffi_name)">
<%call expr="impl_split_style_coord(ident, '%s.mUnit' % gecko_ffi_name, '%s.mValue' % gecko_ffi_name)"></%call>
</%def>

<%def name="impl_corner_style_coord(ident, x_unit_ffi_name, x_union_ffi_name, y_unit_ffi_name, y_union_ffi_name)">
    fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        v.0.width.to_gecko_style_coord(&mut self.gecko.${x_unit_ffi_name},
                                       &mut self.gecko.${x_union_ffi_name});
        v.0.height.to_gecko_style_coord(&mut self.gecko.${y_unit_ffi_name},
                                        &mut self.gecko.${y_union_ffi_name});
    }
    fn copy_${ident}_from(&mut self, other: &Self) {
        use gecko_style_structs::nsStyleUnit::eStyleUnit_Calc;
        debug_assert!(self.gecko.${x_unit_ffi_name} != eStyleUnit_Calc &&
                      self.gecko.${y_unit_ffi_name} != eStyleUnit_Calc,
                      "stylo: Can't yet handle refcounted Calc");
        self.gecko.${x_unit_ffi_name} = other.gecko.${x_unit_ffi_name};
        self.gecko.${x_union_ffi_name} = other.gecko.${x_union_ffi_name};
        self.gecko.${y_unit_ffi_name} = other.gecko.${y_unit_ffi_name};
        self.gecko.${y_union_ffi_name} = other.gecko.${y_union_ffi_name};
    }
</%def>

<%def name="impl_style_struct(style_struct)">
impl ${style_struct.gecko_struct_name} {
    #[allow(dead_code, unused_variables)]
    fn initial() -> Arc<Self> {
        let mut result = Arc::new(${style_struct.gecko_struct_name} { gecko: unsafe { zeroed() } });
        unsafe {
            Gecko_Construct_${style_struct.gecko_ffi_name}(&mut Arc::make_mut(&mut result).gecko);
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
impl Clone for ${style_struct.gecko_ffi_name} {
    fn clone(&self) -> Self {
        unsafe {
            let mut result: Self = zeroed();
            Gecko_CopyConstruct_${style_struct.gecko_ffi_name}(&mut result, self);
            result
        }
    }
}
unsafe impl Send for ${style_struct.gecko_ffi_name} {}
unsafe impl Sync for ${style_struct.gecko_ffi_name} {}
impl HeapSizeOf for ${style_struct.gecko_ffi_name} {
    // Not entirely accurate, but good enough for now.
    fn heap_size_of_children(&self) -> usize { 0 }
}

// FIXME(bholley): Make bindgen generate Debug for all types.
%if style_struct.gecko_ffi_name in "nsStyleBorder nsStyleDisplay nsStyleList nsStyleBackground "\
                                    "nsStyleFont nsStyleSVGReset".split():
impl Debug for ${style_struct.gecko_ffi_name} {
    // FIXME(bholley): Generate this.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GECKO STYLE STRUCT")
    }
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
   # These are currently being shuffled to a different style struct on the gecko side.
   force_stub += ["backface-visibility", "transform-box", "transform-style"]
   # These live in nsStyleImageLayers in gecko. Need to figure out what to do about that.
   force_stub += ["background-repeat", "background-attachment", "background-clip", "background-origin"];
   # These live in an nsFont member in Gecko. Should be straightforward to do manually.
   force_stub += ["font-kerning", "font-stretch", "font-style", "font-variant"]
   # These have unusual representations in gecko.
   force_stub += ["list-style-type", "text-overflow"]
   # Enum class instead of NS_STYLE_...
   force_stub += ["box-sizing"]
   # These are booleans.
   force_stub += ["page-break-after", "page-break-before"]

   # Types used with predefined_type()-defined properties that we can auto-generate.
   predefined_types = {
       "LengthOrPercentage": impl_style_coord,
       "LengthOrPercentageOrAuto": impl_style_coord,
       "LengthOrPercentageOrNone": impl_style_coord,
       "Number": impl_simple,
       "Opacity": impl_simple,
   }

   keyword_longhands = [x for x in longhands if x.keyword and not x.name in force_stub]
   predefined_longhands = [x for x in longhands
                           if x.predefined_type in predefined_types and not x.name in force_stub]
   stub_longhands = [x for x in longhands if x not in keyword_longhands + predefined_longhands]
%>
impl ${style_struct.trait_name} for ${style_struct.gecko_struct_name} {
    /*
     * Manually-Implemented Methods.
     */
    ${caller.body().strip()}

    /*
     * Auto-Generated Methods.
     */
    <%
    for longhand in keyword_longhands:
        impl_keyword(longhand.ident, longhand.gecko_ffi_name, longhand.keyword, longhand.need_clone)
    for longhand in predefined_longhands:
        impl_fn = predefined_types[longhand.predefined_type]
        impl_fn(longhand.ident, longhand.gecko_ffi_name)
    %>

    /*
     * Stubs.
     */
    % for longhand in stub_longhands:
    fn set_${longhand.ident}(&mut self, _: longhands::${longhand.ident}::computed_value::T) {
        println!("stylo: Unimplemented property setter: ${longhand.name}");
    }
    fn copy_${longhand.ident}_from(&mut self, _: &Self) {
        println!("stylo: Unimplemented property setter: ${longhand.name}");
    }
    % if longhand.need_clone:
    fn clone_${longhand.ident}(&self) -> longhands::${longhand.ident}::computed_value::T {
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
<%self:raw_impl_trait style_struct="${next(x for x in data.style_structs if x.trait_name == style_struct_name)}"
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
    def __init__(self, name, index):
        self.x_name = "NS_CORNER_" + name + "_X"
        self.y_name = "NS_CORNER_" + name + "_Y"
        self.ident = name.lower()
        self.x_index = 2 * index
        self.y_index = 2 * index + 1

SIDES = [Side("Top", 0), Side("Right", 1), Side("Bottom", 2), Side("Left", 3)]
CORNERS = [Corner("TOP_LEFT", 0), Corner("TOP_RIGHT", 1), Corner("BOTTOM_RIGHT", 2), Corner("BOTTOM_LEFT", 3)]
%>

#[allow(dead_code)]
fn static_assert() {
    unsafe {
    % for side in SIDES:
        transmute::<_, [u32; ${side.index}]>([1; gecko_style_structs::Side::eSide${side.name} as usize]);
    % endfor
    % for corner in CORNERS:
        transmute::<_, [u32; ${corner.x_index}]>([1; gecko_style_structs::${corner.x_name} as usize]);
        transmute::<_, [u32; ${corner.y_index}]>([1; gecko_style_structs::${corner.y_name} as usize]);
    % endfor
    }
}

<% border_style_keyword = Keyword("border-style",
                                  "none solid double dotted dashed hidden groove ridge inset outset") %>

<% skip_border_longhands = " ".join(["border-{0}-{1}".format(x.ident, y)
                                     for x in SIDES
                                     for y in ["color", "style", "width"]] +
                                    ["border-{0}-radius".format(x.ident.replace("_", "-"))
                                     for x in CORNERS]) %>
<%self:impl_trait style_struct_name="Border"
                  skip_longhands="${skip_border_longhands}"
                  skip_additionals="*">

    % for side in SIDES:
    <% impl_keyword("border_%s_style" % side.ident, "mBorderStyle[%s]" % side.index, border_style_keyword,
                    need_clone=True) %>

    <% impl_color("border_%s_color" % side.ident, "mBorderColor[%s]" % side.index,
                  color_flags_ffi_name="mBorderStyle[%s]" % side.index) %>

    <% impl_app_units("border_%s_width" % side.ident, "mComputedBorder.%s" % side.ident, need_clone=False,
                      round_to_pixels=True) %>

    fn border_${side.ident}_has_nonzero_width(&self) -> bool {
        self.gecko.mComputedBorder.${side.ident} != 0
    }
    % endfor

    % for corner in CORNERS:
    <% impl_corner_style_coord("border_%s_radius" % corner.ident,
                               "mBorderRadius.mUnits[%s]" % corner.x_index,
                               "mBorderRadius.mValues[%s]" % corner.x_index,
                               "mBorderRadius.mUnits[%s]" % corner.y_index,
                               "mBorderRadius.mValues[%s]" % corner.y_index) %>
    % endfor
</%self:impl_trait>

<% skip_margin_longhands = " ".join(["margin-%s" % x.ident for x in SIDES]) %>
<%self:impl_trait style_struct_name="Margin"
                  skip_longhands="${skip_margin_longhands}">

    % for side in SIDES:
    <% impl_split_style_coord("margin_%s" % side.ident,
                              "mMargin.mUnits[%s]" % side.index,
                              "mMargin.mValues[%s]" % side.index) %>
    % endfor
</%self:impl_trait>

<% skip_padding_longhands = " ".join(["padding-%s" % x.ident for x in SIDES]) %>
<%self:impl_trait style_struct_name="Padding"
                  skip_longhands="${skip_padding_longhands}">

    % for side in SIDES:
    <% impl_split_style_coord("padding_%s" % side.ident,
                              "mPadding.mUnits[%s]" % side.index,
                              "mPadding.mValues[%s]" % side.index) %>
    % endfor
</%self:impl_trait>

<% skip_position_longhands = " ".join(x.ident for x in SIDES) %>
<%self:impl_trait style_struct_name="Position"
                  skip_longhands="${skip_position_longhands}">

    % for side in SIDES:
    <% impl_split_style_coord("%s" % side.ident,
                              "mOffset.mUnits[%s]" % side.index,
                              "mOffset.mValues[%s]" % side.index) %>
    % endfor
</%self:impl_trait>

<% skip_outline_longhands = " ".join("outline-color outline-style".split() +
                                     ["-moz-outline-radius-{0}".format(x.ident.replace("_", ""))
                                      for x in CORNERS]) %>
<%self:impl_trait style_struct_name="Outline"
                  skip_longhands="${skip_outline_longhands}"
                  skip_additionals="*">

    <% impl_keyword("outline_style", "mOutlineStyle", border_style_keyword, need_clone=True) %>

    <% impl_color("outline_color", "mOutlineColor", color_flags_ffi_name="mOutlineStyle") %>

    % for corner in CORNERS:
    <% impl_corner_style_coord("_moz_outline_radius_%s" % corner.ident.replace("_", ""),
                               "mOutlineRadius.mUnits[%s]" % corner.x_index,
                               "mOutlineRadius.mValues[%s]" % corner.x_index,
                               "mOutlineRadius.mUnits[%s]" % corner.y_index,
                               "mOutlineRadius.mValues[%s]" % corner.y_index) %>
    % endfor

    fn outline_has_nonzero_width(&self) -> bool {
        self.gecko.mCachedOutlineWidth != 0
    }
</%self:impl_trait>

<%self:impl_trait style_struct_name="Font" skip_longhands="font-size font-weight" skip_additionals="*">

    // FIXME(bholley): Gecko has two different sizes, one of which (mSize) is the
    // actual computed size, and the other of which (mFont.size) is the 'display
    // size' which takes font zooming into account. We don't handle font zooming yet.
    fn set_font_size(&mut self, v: longhands::font_size::computed_value::T) {
        self.gecko.mFont.size = v.0;
        self.gecko.mSize = v.0;
    }
    fn copy_font_size_from(&mut self, other: &Self) {
        self.gecko.mFont.size = other.gecko.mFont.size;
        self.gecko.mSize = other.gecko.mSize;
    }
    fn clone_font_size(&self) -> longhands::font_size::computed_value::T {
        Au(self.gecko.mSize)
    }

    fn set_font_weight(&mut self, v: longhands::font_weight::computed_value::T) {
        self.gecko.mFont.weight = v as u16;
    }
    <%call expr="impl_simple_copy('font_weight', 'mFont.weight')"></%call>

    fn clone_font_weight(&self) -> longhands::font_weight::computed_value::T {
        debug_assert!(self.gecko.mFont.weight >= 100);
        debug_assert!(self.gecko.mFont.weight <= 900);
        debug_assert!(self.gecko.mFont.weight % 10 == 0);
        unsafe { transmute(self.gecko.mFont.weight) }
    }

    // This is used for PartialEq, which we don't implement for gecko style structs.
    fn compute_font_hash(&mut self) {}

</%self:impl_trait>

<%self:impl_trait style_struct_name="Box" skip_longhands="display overflow-y">

    // We manually-implement the |display| property until we get general
    // infrastructure for preffing certain values.
    <% display_keyword = Keyword("display", "inline block inline-block table inline-table table-row-group " +
                                            "table-header-group table-footer-group table-row table-column-group " +
                                            "table-column table-cell table-caption list-item flex none") %>
    <%call expr="impl_keyword('display', 'mDisplay', display_keyword, True)"></%call>

    // overflow-y is implemented as a newtype of overflow-x, so we need special handling.
    // We could generalize this if we run into other newtype keywords.
    <% overflow_x = data.longhands_by_name["overflow-x"] %>
    fn set_overflow_y(&mut self, v: longhands::overflow_y::computed_value::T) {
        use gecko_style_structs as gss;
        use style::properties::longhands::overflow_x::computed_value::T as BaseType;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        self.gecko.mOverflowY = match v.0 {
            % for value in overflow_x.keyword.values_for('gecko'):
                BaseType::${to_rust_ident(value)} => gss::${overflow_x.keyword.gecko_constant(value)} as u8,
            % endfor
        };
    }
    <%call expr="impl_simple_copy('overflow_y', 'mOverflowY')"></%call>
    fn clone_overflow_y(&self) -> longhands::overflow_y::computed_value::T {
        use gecko_style_structs as gss;
        use style::properties::longhands::overflow_x::computed_value::T as BaseType;
        use style::properties::longhands::overflow_y::computed_value::T as NewType;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        match self.gecko.mOverflowY as u32 {
            % for value in overflow_x.keyword.values_for('gecko'):
            gss::${overflow_x.keyword.gecko_constant(value)} => NewType(BaseType::${to_rust_ident(value)}),
            % endfor
            x => panic!("Found unexpected value in style struct for overflow_y property: {}", x),
        }
    }

</%self:impl_trait>

<%self:impl_trait style_struct_name="Background" skip_longhands="background-color" skip_additionals="*">

    <% impl_color("background_color", "mBackgroundColor") %>

</%self:impl_trait>

<%self:impl_trait style_struct_name="List" skip_longhands="list-style-type" skip_additionals="*">

    <% impl_keyword_setter("list_style_type", "__LIST_STYLE_TYPE__",
                           data.longhands_by_name["list-style-type"].keyword) %>
    fn copy_list_style_type_from(&mut self, other: &Self) {
        unsafe {
            Gecko_CopyListStyleTypeFrom(&mut self.gecko, &other.gecko);
        }
    }

</%self:impl_trait>

<%self:impl_trait style_struct_name="Text"
                  skip_longhands="text-decoration-color"
                  skip_additionals="*">

    <% impl_color("text_decoration_color", "mTextDecorationColor",
                  color_flags_ffi_name="mTextDecorationStyle") %>

    fn has_underline(&self) -> bool {
        use gecko_style_structs as gss;
        (self.gecko.mTextDecorationStyle & (gss::NS_STYLE_TEXT_DECORATION_LINE_UNDERLINE as u8)) != 0
    }
    fn has_overline(&self) -> bool {
        use gecko_style_structs as gss;
        (self.gecko.mTextDecorationStyle & (gss::NS_STYLE_TEXT_DECORATION_LINE_OVERLINE as u8)) != 0
    }
    fn has_line_through(&self) -> bool {
        use gecko_style_structs as gss;
        (self.gecko.mTextDecorationStyle & (gss::NS_STYLE_TEXT_DECORATION_LINE_LINE_THROUGH as u8)) != 0
    }
</%self:impl_trait>

<%self:impl_trait style_struct_name="SVG"
                  skip_longhands="flood-color lighting-color stop-color"
                  skip_additionals="*">

    <% impl_color("flood_color", "mFloodColor") %>

    <% impl_color("lighting_color", "mLightingColor") %>

    <% impl_color("stop_color", "mStopColor") %>

</%self:impl_trait>

<%self:impl_trait style_struct_name="Color"
                  skip_longhands="*">

    fn set_color(&mut self, v: longhands::color::computed_value::T) {
        let result = convert_rgba_to_nscolor(&v);
        ${set_gecko_property("mColor", "result")}
    }

    <%call expr="impl_simple_copy('color', 'mColor')"></%call>

    fn clone_color(&self) -> longhands::color::computed_value::T {
        let color = ${get_gecko_property("mColor")} as u32;
        convert_nscolor_to_rgba(color)
    }
</%self:impl_trait>

<%def name="define_ffi_struct_accessor(style_struct)">
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn Servo_GetStyle${style_struct.gecko_name}(computed_values: *mut ServoComputedValues)
  -> *const ${style_struct.gecko_ffi_name} {
    type Helpers = ArcHelpers<ServoComputedValues, GeckoComputedValues>;
    Helpers::with(computed_values, |values| values.get_${style_struct.trait_name_lower}().get_gecko()
                                                as *const ${style_struct.gecko_ffi_name})
}
</%def>

% for style_struct in data.style_structs:
${declare_style_struct(style_struct)}
${impl_style_struct(style_struct)}
% if not style_struct.trait_name in data.manual_style_structs:
<%self:raw_impl_trait style_struct="${style_struct}"></%self:raw_impl_trait>
% endif
${define_ffi_struct_accessor(style_struct)}
% endfor

lazy_static! {
    pub static ref INITIAL_GECKO_VALUES: GeckoComputedValues = GeckoComputedValues {
        % for style_struct in data.style_structs:
           ${style_struct.ident}: ${style_struct.gecko_struct_name}::initial(),
        % endfor
        custom_properties: None,
        shareable: true,
        writing_mode: WritingMode::empty(),
        root_font_size: longhands::font_size::get_initial_value(),
    };
}

// This is a thread-local rather than a lazy static to avoid atomic operations when cascading
// properties.
thread_local!(static CASCADE_PROPERTY: Vec<Option<CascadePropertyFn<GeckoComputedValues>>> = {
    make_cascade_vec::<GeckoComputedValues>()
});
