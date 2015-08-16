/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS and SVG filter support.

use azure::AzFloat;
use azure::azure_hl::{ColorMatrixAttribute, ColorMatrixInput, CompositeInput, DrawTarget};
use azure::azure_hl::{FilterNode, FilterType, LinearTransferAttribute, LinearTransferInput};
use azure::azure_hl::{Matrix5x4, TableTransferAttribute, TableTransferInput};
use azure::azure_hl::{GaussianBlurAttribute, GaussianBlurInput};

use style::computed_values::filter;
use util::geometry::Au;

/// Creates a filter pipeline from a set of CSS filters. Returns the destination end of the filter
/// pipeline and the opacity.
pub fn create_filters(draw_target: &DrawTarget,
                      temporary_draw_target: &DrawTarget,
                      style_filters: &filter::T,
                      accumulated_blur_radius: &mut Au)
                      -> (FilterNode, AzFloat) {
    let mut opacity = 1.0;
    let mut filter = draw_target.create_filter(FilterType::Composite);
    filter.set_input(CompositeInput, &temporary_draw_target.snapshot());
    for style_filter in &style_filters.filters {
        match *style_filter {
            filter::Filter::HueRotate(angle) => {
                let hue_rotate = draw_target.create_filter(FilterType::ColorMatrix);
                let matrix = self::hue_rotate(angle.radians() as AzFloat);
                hue_rotate.set_attribute(ColorMatrixAttribute::Matrix(matrix));
                hue_rotate.set_input(ColorMatrixInput, &filter);
                filter = hue_rotate
            }
            filter::Filter::Opacity(opacity_value) => opacity *= opacity_value as AzFloat,
            filter::Filter::Saturate(amount) => {
                let saturate = draw_target.create_filter(FilterType::ColorMatrix);
                let matrix = self::saturate(amount as AzFloat);
                saturate.set_attribute(ColorMatrixAttribute::Matrix(matrix));
                saturate.set_input(ColorMatrixInput, &filter);
                filter = saturate
            }
            filter::Filter::Sepia(amount) => {
                let sepia = draw_target.create_filter(FilterType::ColorMatrix);
                let matrix = self::sepia(amount as AzFloat);
                sepia.set_attribute(ColorMatrixAttribute::Matrix(matrix));
                sepia.set_input(ColorMatrixInput, &filter);
                filter = sepia
            }
            filter::Filter::Grayscale(amount) => {
                let amount = amount as AzFloat;
                let grayscale = draw_target.create_filter(FilterType::ColorMatrix);
                grayscale.set_attribute(ColorMatrixAttribute::Matrix(self::grayscale(amount)));
                grayscale.set_input(ColorMatrixInput, &filter);
                filter = grayscale
            }
            filter::Filter::Invert(amount) => {
                let amount = amount as AzFloat;
                let invert = draw_target.create_filter(FilterType::TableTransfer);
                invert.set_attribute(TableTransferAttribute::DisableR(false));
                invert.set_attribute(TableTransferAttribute::DisableG(false));
                invert.set_attribute(TableTransferAttribute::DisableB(false));
                invert.set_attribute(TableTransferAttribute::TableR(&[1.0, amount - 1.0]));
                invert.set_attribute(TableTransferAttribute::TableG(&[1.0, amount - 1.0]));
                invert.set_attribute(TableTransferAttribute::TableB(&[1.0, amount - 1.0]));
                invert.set_input(TableTransferInput, &filter);
                filter = invert
            }
            filter::Filter::Brightness(amount) => {
                let amount = amount as AzFloat;
                let brightness = draw_target.create_filter(FilterType::LinearTransfer);
                brightness.set_attribute(LinearTransferAttribute::DisableR(false));
                brightness.set_attribute(LinearTransferAttribute::DisableG(false));
                brightness.set_attribute(LinearTransferAttribute::DisableB(false));
                brightness.set_attribute(LinearTransferAttribute::SlopeR(amount));
                brightness.set_attribute(LinearTransferAttribute::SlopeG(amount));
                brightness.set_attribute(LinearTransferAttribute::SlopeB(amount));
                brightness.set_input(LinearTransferInput, &filter);
                filter = brightness
            }
            filter::Filter::Contrast(amount) => {
                let amount = amount as AzFloat;
                let contrast = draw_target.create_filter(FilterType::LinearTransfer);
                contrast.set_attribute(LinearTransferAttribute::DisableR(false));
                contrast.set_attribute(LinearTransferAttribute::DisableG(false));
                contrast.set_attribute(LinearTransferAttribute::DisableB(false));
                contrast.set_attribute(LinearTransferAttribute::SlopeR(amount));
                contrast.set_attribute(LinearTransferAttribute::SlopeG(amount));
                contrast.set_attribute(LinearTransferAttribute::SlopeB(amount));
                contrast.set_attribute(LinearTransferAttribute::InterceptR(-0.5 * amount + 0.5));
                contrast.set_attribute(LinearTransferAttribute::InterceptG(-0.5 * amount + 0.5));
                contrast.set_attribute(LinearTransferAttribute::InterceptB(-0.5 * amount + 0.5));
                contrast.set_input(LinearTransferInput, &filter);
                filter = contrast
            }
            filter::Filter::Blur(amount) => {
                *accumulated_blur_radius = accumulated_blur_radius.clone() + amount;
                let amount = amount.to_f32_px();
                let blur = draw_target.create_filter(FilterType::GaussianBlur);
                blur.set_attribute(GaussianBlurAttribute::StdDeviation(amount));
                blur.set_input(GaussianBlurInput, &filter);
                filter = blur
            }
        }
    }
    (filter, opacity)
}

/// Determines if we need a temporary draw target for the given set of filters.
pub fn temporary_draw_target_needed_for_style_filters(filters: &filter::T) -> bool {
    for filter in &filters.filters {
        match *filter {
            filter::Filter::Opacity(value) if value == 1.0 => continue,
            _ => return true,
        }
    }
    false
}

// If there is one or more blur filters, we need to know the blur ammount
// to expand the draw target size.
pub fn calculate_accumulated_blur(style_filters: &filter::T) -> Au {
    let mut accum_blur = Au::new(0);
    for style_filter in &style_filters.filters {
        match *style_filter {
            filter::Filter::Blur(amount) => {
                accum_blur = accum_blur.clone() + amount;
            }
            _ => continue,
        }
    }

    accum_blur
}


/// Creates a grayscale 5x4 color matrix per CSS-FILTERS § 12.1.1.
fn grayscale(amount: AzFloat) -> Matrix5x4 {
    Matrix5x4 {
        m11: 0.2126 + 0.7874 * (1.0 - amount),
            m21: 0.7152 - 0.7152 * (1.0 - amount),
            m31: 0.0722 - 0.0722 * (1.0 - amount),
            m41: 0.0,
            m51: 0.0,
        m12: 0.2126 - 0.2126 * (1.0 - amount),
            m22: 0.7152 + 0.2848 * (1.0 - amount),
            m32: 0.0722 - 0.0722 * (1.0 - amount),
            m42: 0.0,
            m52: 0.0,
        m13: 0.2126 - 0.2126 * (1.0 - amount),
            m23: 0.7152 - 0.7152 * (1.0 - amount),
            m33: 0.0722 + 0.9278 * (1.0 - amount),
            m43: 0.0,
            m53: 0.0,
        m14: 0.0, m24: 0.0, m34: 0.0, m44: 1.0, m54: 0.0,
    }
}

/// Creates a 5x4 hue rotation color matrix per CSS-FILTERS § 8.5.
fn hue_rotate(angle: AzFloat) -> Matrix5x4 {
    let (c, s) = (angle.cos(), angle.sin());
    Matrix5x4 {
        m11: 0.213 + c * 0.787 + s * -0.213,
            m21: 0.715 + c * -0.715 + s * -0.715,
            m31: 0.072 + c * -0.072 + s * 0.928,
            m41: 0.0,
            m51: 0.0,
        m12: 0.213 + c * -0.213 + s * 0.143,
            m22: 0.715 + c * 0.285 + s * 0.140,
            m32: 0.072 + c * -0.072 + s * -0.283,
            m42: 0.0,
            m52: 0.0,
        m13: 0.213 + c * -0.213 + s * -0.787,
            m23: 0.715 + c * -0.715 + s * 0.715,
            m33: 0.072 + c * 0.928 + s * 0.072,
            m43: 0.0,
            m53: 0.0,
        m14: 0.0, m24: 0.0, m34: 0.0, m44: 1.0, m54: 0.0,
    }
}

/// Creates a 5x4 saturation color matrix per CSS-FILTERS § 8.5.
fn saturate(amount: AzFloat) -> Matrix5x4 {
    Matrix5x4 {
        m11: 0.213 + 0.787 * amount,
            m21: 0.715 - 0.715 * amount,
            m31: 0.072 - 0.072 * amount,
            m41: 0.0,
            m51: 0.0,
        m12: 0.213 - 0.213 * amount,
            m22: 0.715 + 0.285 * amount,
            m32: 0.072 - 0.072 * amount,
            m42: 0.0,
            m52: 0.0,
        m13: 0.213 - 0.213 * amount,
            m23: 0.715 - 0.715 * amount,
            m33: 0.072 + 0.928 * amount,
            m43: 0.0,
            m53: 0.0,
        m14: 0.0, m24: 0.0, m34: 0.0, m44: 1.0, m54: 0.0,
    }
}

/// Creates a sepia 5x4 color matrix per CSS-FILTERS § 12.1.1.
fn sepia(amount: AzFloat) -> Matrix5x4 {
    Matrix5x4 {
        m11: 0.393 + 0.607 * (1.0 - amount),
            m21: 0.769 - 0.769 * (1.0 - amount),
            m31: 0.189 - 0.189 * (1.0 - amount),
            m41: 0.0,
            m51: 0.0,
        m12: 0.349 - 0.349 * (1.0 - amount),
            m22: 0.686 + 0.314 * (1.0 - amount),
            m32: 0.168 - 0.168 * (1.0 - amount),
            m42: 0.0,
            m52: 0.0,
        m13: 0.272 - 0.272 * (1.0 - amount),
            m23: 0.534 - 0.534 * (1.0 - amount),
            m33: 0.131 + 0.869 * (1.0 - amount),
            m43: 0.0,
            m53: 0.0,
        m14: 0.0, m24: 0.0, m34: 0.0, m44: 1.0, m54: 0.0,
    }
}
