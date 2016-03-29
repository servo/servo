/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
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

% for style_struct in STYLE_STRUCTS:
#[derive(PartialEq, Clone, HeapSizeOf, Debug)]
pub struct Gecko${style_struct.name};
impl T${style_struct.name} for Gecko${style_struct.name} {
    % for longhand in style_struct.longhands:
    fn set_${longhand.ident}(&mut self, _: longhands::${longhand.ident}::computed_value::T) {
        unimplemented!()
    }
    fn copy_${longhand.ident}_from(&mut self, _: &Self) {
        unimplemented!()
    }
    % endfor
    % if style_struct.name == "Animation":
    fn transition_count(&self) -> usize {
        unimplemented!()
    }
    % elif style_struct.name == "Border":
    % for side in ["top", "right", "bottom", "left"]:
    fn border_${side}_is_none_or_hidden_and_has_nonzero_width(&self) -> bool {
        unimplemented!()
    }
    % endfor
    % elif style_struct.name == "Box":
    fn clone_display(&self) -> longhands::display::computed_value::T {
        unimplemented!()
    }
    fn clone_position(&self) -> longhands::position::computed_value::T {
        unimplemented!()
    }
    fn is_floated(&self) -> bool {
        unimplemented!()
    }
    fn overflow_x_is_visible(&self) -> bool {
        unimplemented!()
    }
    fn overflow_y_is_visible(&self) -> bool {
        unimplemented!()
    }
    % elif style_struct.name == "Color":
    fn clone_color(&self) -> longhands::color::computed_value::T {
        unimplemented!()
    }
    % elif style_struct.name == "Font":
    fn clone_font_size(&self) -> longhands::font_size::computed_value::T {
        unimplemented!()
    }
    fn clone_font_weight(&self) -> longhands::font_weight::computed_value::T {
        unimplemented!()
    }
    fn compute_font_hash(&mut self) {
        unimplemented!()
    }
    % elif style_struct.name == "InheritedBox":
    fn clone_direction(&self) -> longhands::direction::computed_value::T {
        unimplemented!()
    }
    fn clone_writing_mode(&self) -> longhands::writing_mode::computed_value::T {
        unimplemented!()
    }
    fn clone_text_orientation(&self) -> longhands::text_orientation::computed_value::T {
        unimplemented!()
    }
    % elif style_struct.name == "InheritedText":
    fn clone__servo_text_decorations_in_effect(&self) ->
        longhands::_servo_text_decorations_in_effect::computed_value::T {
        unimplemented!()
    }
    % elif style_struct.name == "Outline":
    fn outline_is_none_or_hidden_and_has_nonzero_width(&self) -> bool {
        unimplemented!()
    }
    % elif style_struct.name == "Text":
    fn has_underline(&self) -> bool {
        unimplemented!()
    }
    fn has_overline(&self) -> bool {
        unimplemented!()
    }
    fn has_line_through(&self) -> bool {
        unimplemented!()
    }
    % endif

}

% endfor
