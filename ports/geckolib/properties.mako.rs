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
%if style_struct.gecko_ffi_name:
use gecko_style_structs::${style_struct.gecko_ffi_name};
use bindings::Gecko_Construct_${style_struct.gecko_ffi_name};
use bindings::Gecko_CopyConstruct_${style_struct.gecko_ffi_name};
use bindings::Gecko_Destroy_${style_struct.gecko_ffi_name};
% endif
% endfor
use gecko_style_structs;
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
% if style_struct.gecko_ffi_name:
pub struct ${style_struct.gecko_struct_name} {
    gecko: ${style_struct.gecko_ffi_name},
}
% else:
pub struct ${style_struct.gecko_struct_name};
% endif
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

<%def name="impl_keyword(ident, gecko_ffi_name, keyword, need_clone)">
<%call expr="impl_keyword_setter(ident, gecko_ffi_name, keyword)"></%call>
<%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
%if need_clone:
<%call expr="impl_keyword_clone(ident, gecko_ffi_name, keyword)"></%call>
% endif
</%def>

<%def name="impl_app_units(ident, gecko_ffi_name, need_clone)">
    fn set_${ident}(&mut self, v: longhands::${ident}::computed_value::T) {
        self.gecko.${gecko_ffi_name} = v.0;
    }
<%call expr="impl_simple_copy(ident, gecko_ffi_name)"></%call>
%if need_clone:
    fn clone_${ident}(&self) -> longhands::${ident}::computed_value::T {
        Au(self.gecko.${gecko_ffi_name})
    }
% endif
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
}
%if style_struct.gecko_ffi_name:
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
%if style_struct.gecko_ffi_name in "nsStyleBorder nsStylePosition nsStyleDisplay nsStyleList nsStyleBackground "\
                                    "nsStyleFont nsStyleEffects nsStyleSVGReset".split():
impl Debug for ${style_struct.gecko_ffi_name} {
    // FIXME(bholley): Generate this.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GECKO STYLE STRUCT")
    }
}
%endif
%endif
</%def>

<%def name="raw_impl_trait(style_struct, skip_longhands='', skip_additionals='')">
<%
   longhands = [x for x in style_struct.longhands if not x.name in skip_longhands.split()]

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
   # Inconsistent constant naming in gecko
   force_stub += ["unicode-bidi", "text-transform"]
   # These are booleans.
   force_stub += ["page-break-after", "page-break-before"]

   keyword_longhands = [x for x in longhands if x.keyword and not x.name in force_stub]
   stub_longhands = [x for x in longhands if x not in keyword_longhands]
%>
impl ${style_struct.trait_name} for ${style_struct.gecko_struct_name} {
    /*
     * Manually-Implemented Methods.
     */
    ${caller.body().strip()}

    /*
     * Auto-Generated Methods.
     */
    % for longhand in keyword_longhands:
    <%call expr="impl_keyword(longhand.ident, longhand.gecko_ffi_name, longhand.keyword, longhand.need_clone)"></%call>
    % endfor

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

SIDES = [Side("Top", 0), Side("Right", 1), Side("Bottom", 2), Side("Left", 3)]

%>

#[allow(dead_code)]
fn static_assert() {
    unsafe {
    % for side in SIDES:
        transmute::<_, [u32; ${side.index}]>([1; gecko_style_structs::Side::eSide${side.name} as usize]);
    % endfor
    }
}

<% border_style_keyword = Keyword("border-style",
                                  "none solid double dotted dashed hidden groove ridge inset outset") %>

<%
skip_border_longhands = ""
for side in SIDES:
    skip_border_longhands += "border-{0}-style border-{0}-width ".format(side.ident)
%>

<%self:impl_trait style_struct_name="Border"
                  skip_longhands="${skip_border_longhands}"
                  skip_additionals="*">

    % for side in SIDES:
    <% impl_keyword("border_%s_style" % side.ident, "mBorderStyle[%s]" % side.index, border_style_keyword,
                    need_clone=True) %>

    <% impl_app_units("border_%s_width" % side.ident, "mComputedBorder.%s" % side.ident, need_clone=False) %>

    fn border_${side.ident}_has_nonzero_width(&self) -> bool {
        self.gecko.mComputedBorder.${side.ident} != 0
    }
    % endfor
</%self:impl_trait>

<%self:impl_trait style_struct_name="Outline"
                  skip_longhands="outline-style"
                  skip_additionals="*">

    <% impl_keyword("outline_style", "mOutlineStyle", border_style_keyword, need_clone=True) %>

    fn outline_has_nonzero_width(&self) -> bool {
        self.gecko.mCachedOutlineWidth != 0
    }
</%self:impl_trait>

<%self:impl_trait style_struct_name="Font" skip_longhands="font-size" skip_additionals="*">

    // FIXME(bholley): This doesn't handle zooming properly.
    <% impl_app_units("font_size", "mSize", need_clone=True) %>

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

% for style_struct in data.style_structs:
${declare_style_struct(style_struct)}
${impl_style_struct(style_struct)}
% if not style_struct.trait_name in data.manual_style_structs:
<%self:raw_impl_trait style_struct="${style_struct}"></%self:raw_impl_trait>
% endif
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
