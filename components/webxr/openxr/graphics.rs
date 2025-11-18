/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::{Size2D, UnknownUnit};
use openxr::{ExtensionSet, FrameStream, FrameWaiter, Graphics, Instance, Session, SystemId};
use surfman::{
    Context as SurfmanContext, Device as SurfmanDevice, Error as SurfmanError, SurfaceTexture,
};
use webxr_api::Error;

pub enum GraphicsProvider {}

pub trait GraphicsProviderMethods<G: Graphics> {
    fn enable_graphics_extensions(exts: &mut ExtensionSet);
    fn pick_format(formats: &[u32]) -> u32;
    fn create_session(
        device: &SurfmanDevice,
        instance: &Instance,
        system: SystemId,
    ) -> Result<(Session<G>, FrameWaiter, FrameStream<G>), Error>;
    fn surface_texture_from_swapchain_texture(
        image: <G as Graphics>::SwapchainImage,
        device: &mut SurfmanDevice,
        context: &mut SurfmanContext,
        size: &Size2D<i32, UnknownUnit>,
    ) -> Result<SurfaceTexture, SurfmanError>;
}
