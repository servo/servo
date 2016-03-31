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
    fn initial() -> Self {
% if style_struct.gecko_ffi_name:
        let mut result = ${style_struct.gecko_struct_name} { gecko: unsafe { zeroed() } };
        unsafe {
            Gecko_Construct_${style_struct.gecko_ffi_name}(&mut result.gecko);
        }
        result
% else:
        ${style_struct.gecko_struct_name}
% endif
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
impl T${style_struct.trait_name} for ${style_struct.gecko_struct_name} {
    /*
     * Manually-Implemented Methods.
     */
    ${caller.body().strip()}

    /*
     * Auto-Generated Methods.
     */
    <% longhands = [x for x in style_struct.longhands
                    if not (skip_longhands and x.name in skip_longhands)] %>
    % for longhand in longhands:
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
           ${style_struct.ident}: Arc::new(${style_struct.gecko_struct_name}::initial()),
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
