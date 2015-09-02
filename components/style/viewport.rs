/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::scale_factor::ScaleFactor;
use euclid::size::{Size2D, TypedSize2D};
use properties::longhands;
use util::geometry::{Au, ViewportPx};
use style_traits::{Length, Orientation,
                   LengthOrPercentageOrAuto, ViewportDescriptor, UserZoom,
                   ViewportConstraints, ViewportRule};

pub trait MaybeNew {
    fn maybe_new(initial_viewport: TypedSize2D<ViewportPx, f32>,
                 rule: &ViewportRule)
                 -> Option<ViewportConstraints>;
}

impl MaybeNew for ViewportConstraints {
    fn maybe_new(initial_viewport: TypedSize2D<ViewportPx, f32>,
                     rule: &ViewportRule)
                     -> Option<ViewportConstraints>
    {
        use num::{Float, ToPrimitive};
        use std::cmp;

        if rule.declarations.is_empty() {
            return None
        }

        let mut min_width = None;
        let mut max_width = None;

        let mut min_height = None;
        let mut max_height = None;

        let mut initial_zoom = None;
        let mut min_zoom = None;
        let mut max_zoom = None;

        let mut user_zoom = UserZoom::Zoom;
        let mut orientation = Orientation::Auto;

        // collapse the list of declarations into descriptor values
        for declaration in &rule.declarations {
            match declaration.descriptor {
                ViewportDescriptor::MinWidth(value) => min_width = Some(value),
                ViewportDescriptor::MaxWidth(value) => max_width = Some(value),

                ViewportDescriptor::MinHeight(value) => min_height = Some(value),
                ViewportDescriptor::MaxHeight(value) => max_height = Some(value),

                ViewportDescriptor::Zoom(value) => initial_zoom = value.to_f32(),
                ViewportDescriptor::MinZoom(value) => min_zoom = value.to_f32(),
                ViewportDescriptor::MaxZoom(value) => max_zoom = value.to_f32(),

                ViewportDescriptor::UserZoom(value) => user_zoom = value,
                ViewportDescriptor::Orientation(value) => orientation = value
            }
        }

        // TODO: return `None` if all descriptors are either absent or initial value

        macro_rules! choose {
            ($op:ident, $opta:expr, $optb:expr) => {
                match ($opta, $optb) {
                    (None, None) => None,
                    (a, None) => a.clone(),
                    (None, b) => b.clone(),
                    (a, b) => Some(a.clone().unwrap().$op(b.clone().unwrap())),
                }
            }
        }
        macro_rules! min {
            ($opta:expr, $optb:expr) => {
                choose!(min, $opta, $optb)
            }
        }
        macro_rules! max {
            ($opta:expr, $optb:expr) => {
                choose!(max, $opta, $optb)
            }
        }

        // DEVICE-ADAPT § 6.2.1 Resolve min-zoom and max-zoom values
        if min_zoom.is_some() && max_zoom.is_some() {
            max_zoom = Some(min_zoom.clone().unwrap().max(max_zoom.unwrap()))
        }

        // DEVICE-ADAPT § 6.2.2 Constrain zoom value to the [min-zoom, max-zoom] range
        if initial_zoom.is_some() {
            initial_zoom = max!(min_zoom, min!(max_zoom, initial_zoom));
        }

        // DEVICE-ADAPT § 6.2.3 Resolve non-auto lengths to pixel lengths
        //
        // Note: DEVICE-ADAPT § 5. states that relative length values are
        // resolved against initial values
        let initial_viewport = Size2D::new(Au::from_f32_px(initial_viewport.width.get()),
                                           Au::from_f32_px(initial_viewport.height.get()));

        macro_rules! to_pixel_length {
            ($value:ident, $dimension:ident) => {
                if let Some($value) = $value {
                    match $value {
                        LengthOrPercentageOrAuto::Length(ref value) => Some(match value {
                            &Length::Absolute(length) => length,
                            &Length::FontRelative(length) => {
                                let initial_font_size = longhands::font_size::get_initial_value();
                                length.to_computed_value(initial_font_size, initial_font_size)
                            }
                            &Length::ViewportPercentage(length) =>
                                length.to_computed_value(initial_viewport),
                            _ => unreachable!()
                        }),
                        LengthOrPercentageOrAuto::Percentage(value) =>
                            Some(initial_viewport.$dimension.scale_by(value)),
                        LengthOrPercentageOrAuto::Auto => None,
                    }
                } else {
                    None
                }
            }
        }

        let min_width = to_pixel_length!(min_width, width);
        let max_width = to_pixel_length!(max_width, width);
        let min_height = to_pixel_length!(min_height, height);
        let max_height = to_pixel_length!(max_height, height);

        // DEVICE-ADAPT § 6.2.4 Resolve initial width and height from min/max descriptors
        macro_rules! resolve {
            ($min:ident, $max:ident, $initial:expr) => {
                if $min.is_some() || $max.is_some() {
                    let max = match $max {
                        Some(max) => cmp::min(max, $initial),
                        None => $initial
                    };

                    Some(match $min {
                        Some(min) => cmp::max(min, max),
                        None => max
                    })
                } else {
                    None
                };
            }
        }

        let width = resolve!(min_width, max_width, initial_viewport.width);
        let height = resolve!(min_height, max_height, initial_viewport.height);

        // DEVICE-ADAPT § 6.2.5 Resolve width value
        let width = if width.is_none() && height.is_none() {
            Some(initial_viewport.width)
        } else {
            width
        };

        let width = width.unwrap_or_else(|| match initial_viewport.height {
            Au(0) => initial_viewport.width,
            initial_height => {
                let ratio = initial_viewport.width.to_f32_px() / initial_height.to_f32_px();
                Au::from_f32_px(height.clone().unwrap().to_f32_px() * ratio)
            }
        });

        // DEVICE-ADAPT § 6.2.6 Resolve height value
        let height = height.unwrap_or_else(|| match initial_viewport.width {
            Au(0) => initial_viewport.height,
            initial_width => {
                let ratio = initial_viewport.height.to_f32_px() / initial_width.to_f32_px();
                Au::from_f32_px(width.to_f32_px() * ratio)
            }
        });

        Some(ViewportConstraints {
            size: Size2D::typed(width.to_f32_px(), height.to_f32_px()),

            // TODO: compute a zoom factor for 'auto' as suggested by DEVICE-ADAPT § 10.
            initial_zoom: ScaleFactor::new(initial_zoom.unwrap_or(1.)),
            min_zoom: min_zoom.map(ScaleFactor::new),
            max_zoom: max_zoom.map(ScaleFactor::new),

            user_zoom: user_zoom,
            orientation: orientation
        })
    }
}
