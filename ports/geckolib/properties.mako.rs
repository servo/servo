/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// STYLE_STRUCTS comes from components/style/properties.mako.rs; see build.rs for more details.

use app_units::Au;
% for style_struct in STYLE_STRUCTS:
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
use std::mem::zeroed;
use std::sync::Arc;
use style::custom_properties::ComputedValuesMap;
use style::logical_geometry::WritingMode;
use style::properties::{CascadePropertyFn, ServoComputedValues, ComputedValues};
use style::properties::longhands;
use style::properties::make_cascade_vec;
use style::properties::style_struct_traits::*;

#[derive(Clone)]
pub struct GeckoComputedValues {
    % for style_struct in STYLE_STRUCTS:
    ${style_struct.ident}: Arc<${style_struct.gecko_struct_name}>,
    % endfor

    custom_properties: Option<Arc<ComputedValuesMap>>,
    shareable: bool,
    pub writing_mode: WritingMode,
    pub root_font_size: Au,
}

impl ComputedValues for GeckoComputedValues {
% for style_struct in STYLE_STRUCTS:
    type Concrete${style_struct.trait_name} = ${style_struct.gecko_struct_name};
% endfor

    // These will go away, and we will never implement them.
    fn as_servo<'a>(&'a self) -> &'a ServoComputedValues { unimplemented!() }
    fn as_servo_mut<'a>(&'a mut self) -> &'a mut ServoComputedValues { unimplemented!() }

    fn new(custom_properties: Option<Arc<ComputedValuesMap>>,
           shareable: bool,
           writing_mode: WritingMode,
           root_font_size: Au,
            % for style_struct in STYLE_STRUCTS:
           ${style_struct.ident}: Arc<${style_struct.gecko_struct_name}>,
            % endfor
    ) -> Self {
        GeckoComputedValues {
            custom_properties: custom_properties,
            shareable: shareable,
            writing_mode: writing_mode,
            root_font_size: root_font_size,
            % for style_struct in STYLE_STRUCTS:
            ${style_struct.ident}: ${style_struct.ident},
            % endfor
        }
    }

    fn initial_values() -> &'static Self { &*INITIAL_GECKO_VALUES }

    fn do_cascade_property<F: FnOnce(&Vec<Option<CascadePropertyFn<Self>>>)>(f: F) {
        CASCADE_PROPERTY.with(|x| f(x));
    }

    % for style_struct in STYLE_STRUCTS:
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

    #[inline]
    fn is_multicol(&self) -> bool { unimplemented!() }
}

<%def name="declare_style_struct(style_struct)">
#[derive(Clone, HeapSizeOf, Debug)]
#[no_move]
% if style_struct.gecko_ffi_name:
pub struct ${style_struct.gecko_struct_name} {
    gecko: ${style_struct.gecko_ffi_name},
}
% else:
pub struct ${style_struct.gecko_struct_name};
% endif
</%def>

<%def name="impl_style_struct(style_struct)">
impl ${style_struct.gecko_struct_name} {
    #[allow(dead_code, unused_variables)]
    fn initial() -> Arc<Self> {
        // Some Gecko style structs have AutoTArray members, which have internal pointers and are
        // thus MOZ_NON_MEMMOVABLE. Since Rust is generally a very move-happy language, we need to
        // be very careful that nsStyle* structs are never moved after they are constructed.
        //
        // By annotating the structs [no_move], we can get the |rust-tenacious| linter to trigger
        // an error on any semantic moves. But we don't have a great way of telling LLVM to
        // allocate our new object directly on the heap without using a temporary. So to do that
        // (and also please tenacious), we pass zeroed memory into the Arc constructor, and _then_
        // use make_mut to get a reference to pass to the Gecko constructor. Since the refcount is
        // guaranteed to be 1, make_mut will always pass us a direct reference instead of taking
        // the copy-on-write path.
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
impl Debug for ${style_struct.gecko_ffi_name} {
    // FIXME(bholley): Generate this.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GECKO STYLE STRUCT")
    }
}
%endif
</%def>

<%def name="raw_impl_trait(style_struct, skip_longhands=None, skip_additionals=None)">
<%
   longhands = [x for x in style_struct.longhands
                if not (skip_longhands and x.name in skip_longhands)]

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
   force_stub += ["unicode-bidi"]
   # Need to figure out why servo has sideways-left computed value and gecko doesn't
   force_stub += ["text-orientation"]
   # Automatic mapping generates NS_STYLE_TEXT_DECORATION_STYLE__MOZ_NONE instead of
   # NS_STYLE_TEXT_DECORATION_STYLE__NONE
   force_stub += ["text-decoration-style"]
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
    fn set_${longhand.ident}(&mut self, v: longhands::${longhand.ident}::computed_value::T) {
        use gecko_style_structs as gss;
        use style::properties::longhands::${longhand.ident}::computed_value::T as Keyword;
        // FIXME(bholley): Align binary representations and ditch |match| for cast + static_asserts
        self.gecko.${longhand.gecko_ffi_name} = match v {
            % for value in longhand.keyword.values_for('gecko'):
                Keyword::${to_rust_ident(value)} => gss::${longhand.keyword.gecko_constant(value)} as u8,
            % endfor
        };
    }
    fn copy_${longhand.ident}_from(&mut self, other: &Self) {
        self.gecko.${longhand.gecko_ffi_name} = other.gecko.${longhand.gecko_ffi_name};
    }
    % endfor

    /*
     * Stubs.
     */
    % for longhand in stub_longhands:
    fn set_${longhand.ident}(&mut self, _: longhands::${longhand.ident}::computed_value::T) {
        unimplemented!()
    }
    fn copy_${longhand.ident}_from(&mut self, _: &Self) {
        unimplemented!()
    }
    % endfor
    <% additionals = [x for x in style_struct.additional_methods
                      if not (skip_additionals and x.name in skip_additionals)] %>
    % for additional in additionals:
    ${additional.stub()}
    % endfor
}
</%def>

<%! MANUAL_STYLE_STRUCTS = [] %>
<%def name="impl_trait(style_struct_name, skip_longhands=None, skip_additionals=None)">
<%self:raw_impl_trait style_struct="${next(x for x in STYLE_STRUCTS if x.trait_name == style_struct_name)}"
                      skip_longhands="${skip_longhands}" skip_additionals="${skip_additionals}">
${caller.body()}
</%self:raw_impl_trait>
<% MANUAL_STYLE_STRUCTS.append(style_struct_name) %>
</%def>

// Proof-of-concept for a style struct with some manually-implemented methods. We add
// the manually-implemented methods to skip_longhands and skip_additionals, and the
// infrastructure auto-generates everything not in those lists. This allows us to
// iteratively implement more and more methods.
<%self:impl_trait style_struct_name="Border"
                  skip_longhands="${['border-left-color', 'border-left-style']}"
                  skip_additionals="${['border_bottom_is_none_or_hidden_and_has_nonzero_width']}">
    fn set_border_left_color(&mut self, _: longhands::border_left_color::computed_value::T) {
        unimplemented!()
    }
    fn copy_border_left_color_from(&mut self, _: &Self) {
        unimplemented!()
    }
    fn set_border_left_style(&mut self, _: longhands::border_left_style::computed_value::T) {
        unimplemented!()
    }
    fn copy_border_left_style_from(&mut self, _: &Self) {
        unimplemented!()
    }
    fn border_bottom_is_none_or_hidden_and_has_nonzero_width(&self) -> bool {
        unimplemented!()
    }
</%self:impl_trait>

% for style_struct in STYLE_STRUCTS:
${declare_style_struct(style_struct)}
${impl_style_struct(style_struct)}
% if not style_struct.trait_name in MANUAL_STYLE_STRUCTS:
<%self:raw_impl_trait style_struct="${style_struct}"></%self:raw_impl_trait>
% endif
% endfor

lazy_static! {
    pub static ref INITIAL_GECKO_VALUES: GeckoComputedValues = GeckoComputedValues {
        % for style_struct in STYLE_STRUCTS:
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
