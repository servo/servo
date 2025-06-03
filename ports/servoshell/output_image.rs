/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use euclid::Point2D;
use image::{DynamicImage, ImageFormat};
use log::error;
use servo::RenderingContext;
use servo::webrender_api::units::DeviceIntRect;

use crate::prefs::ServoShellPreferences;

// This needs to be done before presenting(), because `ReneringContext::read_to_image` reads
// from the back buffer.
pub(crate) fn save_output_image_if_necessary<T>(
    prefs: &ServoShellPreferences,
    rendering_context: &Rc<T>,
) where
    T: RenderingContext + ?Sized,
{
    let Some(output_path) = prefs.output_image_path.as_ref() else {
        return;
    };

    let size = rendering_context.size2d().to_i32();
    let viewport_rect = DeviceIntRect::from_origin_and_size(Point2D::origin(), size);
    let Some(image) = rendering_context.read_to_image(viewport_rect) else {
        error!("Failed to read output image.");
        return;
    };

    let image_format = ImageFormat::from_path(output_path).unwrap_or(ImageFormat::Png);
    if let Err(error) = DynamicImage::ImageRgba8(image).save_with_format(output_path, image_format)
    {
        error!("Failed to save {output_path}: {error}.");
    }
}
