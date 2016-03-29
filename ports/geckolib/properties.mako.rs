/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
% for style_struct in STYLE_STRUCTS:
%if style_struct.gecko_name:
use gecko_style_structs::${style_struct.gecko_name};
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
use style::properties::style_struct_traits::*;

#[derive(Clone)]
pub struct GeckoComputedValues {
    % for style_struct in STYLE_STRUCTS:
    ${style_struct.ident}: Arc<Gecko${style_struct.name}>,
    % endfor

    custom_properties: Option<Arc<ComputedValuesMap>>,
    shareable: bool,
    pub writing_mode: WritingMode,
    pub root_font_size: Au,
}

impl ComputedValues for GeckoComputedValues {
% for style_struct in STYLE_STRUCTS:
    type Concrete${style_struct.name} = Gecko${style_struct.name};
% endfor

    // These will go away, and we will never implement them.
    fn as_servo<'a>(&'a self) -> &'a ServoComputedValues { unimplemented!() }
    fn as_servo_mut<'a>(&'a mut self) -> &'a mut ServoComputedValues { unimplemented!() }

    fn new(custom_properties: Option<Arc<ComputedValuesMap>>,
           shareable: bool,
           writing_mode: WritingMode,
           root_font_size: Au,
            % for style_struct in STYLE_STRUCTS:
           ${style_struct.ident}: Arc<Gecko${style_struct.name}>,
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

    fn initial_values() -> &'static Self { unimplemented!() }

    fn do_cascade_property<F: FnOnce(&Vec<Option<CascadePropertyFn<Self>>>)>(_: F) {
        unimplemented!()
    }

    % for style_struct in STYLE_STRUCTS:
    #[inline]
    fn clone_${style_struct.name.lower()}(&self) -> Arc<Self::Concrete${style_struct.name}> {
        self.${style_struct.ident}.clone()
    }
    #[inline]
    fn get_${style_struct.name.lower()}<'a>(&'a self) -> &'a Self::Concrete${style_struct.name} {
        &self.${style_struct.ident}
    }
    #[inline]
    fn mutate_${style_struct.name.lower()}<'a>(&'a mut self) -> &'a mut Self::Concrete${style_struct.name} {
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
% if style_struct.gecko_name:
pub struct Gecko${style_struct.name} {
    gecko: ${style_struct.gecko_name},
}
% else:
pub struct Gecko${style_struct.name};
% endif
</%def>

<%def name="impl_style_struct(style_struct)">
impl Gecko${style_struct.name} {
    #[allow(dead_code, unused_variables)]
    fn initial() -> Self {
% if style_struct.gecko_name:
        let result = Gecko${style_struct.name} { gecko: unsafe { zeroed() } };
        panic!("Need to invoke Gecko placement new");
% else:
        Gecko${style_struct.name}
% endif
    }
}
%if style_struct.gecko_name:
impl Drop for Gecko${style_struct.name} {
    fn drop(&mut self) {
        panic!("Need to invoke Gecko destructor");
    }
}
impl Clone for ${style_struct.gecko_name} {
    fn clone(&self) -> Self {
        panic!("Need to invoke Gecko copy constructor");
    }
}
unsafe impl Send for ${style_struct.gecko_name} {}
unsafe impl Sync for ${style_struct.gecko_name} {}
impl HeapSizeOf for ${style_struct.gecko_name} {
    // Not entirely accurate, but good enough for now.
    fn heap_size_of_children(&self) -> usize { 0 }
}
impl Debug for ${style_struct.gecko_name} {
    // FIXME(bholley): Generate this.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GECKO STYLE STRUCT")
    }
}
%endif
</%def>

% for style_struct in STYLE_STRUCTS:
${declare_style_struct(style_struct)}
${impl_style_struct(style_struct)}
impl T${style_struct.name} for Gecko${style_struct.name} {
    % for longhand in style_struct.longhands:
    fn set_${longhand.ident}(&mut self, _: longhands::${longhand.ident}::computed_value::T) {
        unimplemented!()
    }
    fn copy_${longhand.ident}_from(&mut self, _: &Self) {
        unimplemented!()
    }
    % endfor
    % for additional in style_struct.additional_methods:
    ${additional.stub()}
    % endfor
}

% endfor
