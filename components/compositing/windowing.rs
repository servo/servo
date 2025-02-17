/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Abstract windowing methods. The concrete implementations of these can be found in `platform/`.

use std::fmt::Debug;

use embedder_traits::{EventLoopWaker, MouseButton};
use euclid::Scale;
use net::protocols::ProtocolRegistry;
use servo_geometry::{DeviceIndependentIntRect, DeviceIndependentIntSize, DeviceIndependentPixel};
use webrender_api::units::{DeviceIntRect, DeviceIntSize, DevicePixel, DevicePoint};

#[derive(Clone)]
pub enum MouseWindowEvent {
    Click(MouseButton, DevicePoint),
    MouseDown(MouseButton, DevicePoint),
    MouseUp(MouseButton, DevicePoint),
}

/// Various debug and profiling flags that WebRender supports.
#[derive(Clone)]
pub enum WebRenderDebugOption {
    Profiler,
    TextureCacheDebug,
    RenderTargetDebug,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationState {
    Idle,
    Animating,
}

// TODO: this trait assumes that the window is responsible
// for creating the GL context, making it current, buffer
// swapping, etc. Really that should all be done by surfman.
pub trait WindowMethods {
    /// Get the coordinates of the native window, the screen and the framebuffer.
    fn get_coordinates(&self) -> EmbedderCoordinates;
    /// Set whether the application is currently animating.
    /// Typically, when animations are active, the window
    /// will want to avoid blocking on UI events, and just
    /// run the event loop at the vsync interval.
    fn set_animation_state(&self, _state: AnimationState);
}

pub trait EmbedderMethods {
    /// Returns a thread-safe object to wake up the window's event loop.
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker>;

    #[cfg(feature = "webxr")]
    /// Register services with a WebXR Registry.
    fn register_webxr(
        &mut self,
        _: &mut webxr::MainThreadRegistry,
        _: embedder_traits::EmbedderProxy,
    ) {
    }

    /// Returns the user agent string to report in network requests.
    fn get_user_agent_string(&self) -> Option<String> {
        None
    }

    /// Returns the version string of this embedder.
    fn get_version_string(&self) -> Option<String> {
        None
    }

    /// Returns the protocol handlers implemented by that embedder.
    /// They will be merged with the default internal ones.
    fn get_protocol_handlers(&self) -> ProtocolRegistry {
        ProtocolRegistry::default()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EmbedderCoordinates {
    /// The pixel density of the display.
    pub hidpi_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    /// Size of the screen.
    pub screen_size: DeviceIndependentIntSize,
    /// Size of the available screen space (screen without toolbars and docks).
    pub available_screen_size: DeviceIndependentIntSize,
    /// Position and size of the native window.
    pub window_rect: DeviceIndependentIntRect,
    /// Size of the GL buffer in the window.
    pub framebuffer: DeviceIntSize,
    /// Coordinates of the document within the framebuffer.
    pub viewport: DeviceIntRect,
}

impl EmbedderCoordinates {
    /// Get the unflipped viewport rectangle for use with the WebRender API.
    pub fn get_viewport(&self) -> DeviceIntRect {
        self.viewport
    }

    /// Flip the given rect.
    /// This should be used when drawing directly to the framebuffer with OpenGL commands.
    pub fn flip_rect(&self, rect: &DeviceIntRect) -> DeviceIntRect {
        let mut result = *rect;
        let min_y = self.framebuffer.height - result.max.y;
        let max_y = self.framebuffer.height - result.min.y;
        result.min.y = min_y;
        result.max.y = max_y;
        result
    }

    /// Get the flipped viewport rectangle.
    /// This should be used when drawing directly to the framebuffer with OpenGL commands.
    pub fn get_flipped_viewport(&self) -> DeviceIntRect {
        self.flip_rect(&self.get_viewport())
    }
}

#[cfg(test)]
mod test {
    use euclid::{Box2D, Point2D, Scale, Size2D};
    use webrender_api::units::DeviceIntRect;

    use super::EmbedderCoordinates;

    #[test]
    fn test() {
        let screen_size = Size2D::new(1080, 720);
        let viewport = Box2D::from_origin_and_size(Point2D::zero(), Size2D::new(800, 600));
        let window_rect = Box2D::from_origin_and_size(Point2D::zero(), Size2D::new(800, 600));
        let coordinates = EmbedderCoordinates {
            hidpi_factor: Scale::new(1.),
            screen_size,
            available_screen_size: screen_size,
            window_rect,
            framebuffer: viewport.size(),
            viewport,
        };

        // Check if viewport conversion is correct.
        let viewport = DeviceIntRect::new(Point2D::new(0, 0), Point2D::new(800, 600));
        assert_eq!(coordinates.get_viewport(), viewport);
        assert_eq!(coordinates.get_flipped_viewport(), viewport);

        // Check rects with different y positions inside the viewport.
        let rect1 = DeviceIntRect::new(Point2D::new(0, 0), Point2D::new(800, 400));
        let rect2 = DeviceIntRect::new(Point2D::new(0, 100), Point2D::new(800, 600));
        let rect3 = DeviceIntRect::new(Point2D::new(0, 200), Point2D::new(800, 500));
        assert_eq!(
            coordinates.flip_rect(&rect1),
            DeviceIntRect::new(Point2D::new(0, 200), Point2D::new(800, 600))
        );
        assert_eq!(
            coordinates.flip_rect(&rect2),
            DeviceIntRect::new(Point2D::new(0, 0), Point2D::new(800, 500))
        );
        assert_eq!(
            coordinates.flip_rect(&rect3),
            DeviceIntRect::new(Point2D::new(0, 100), Point2D::new(800, 400))
        );

        // Check rects with different x positions.
        let rect1 = DeviceIntRect::new(Point2D::new(0, 0), Point2D::new(700, 400));
        let rect2 = DeviceIntRect::new(Point2D::new(100, 100), Point2D::new(800, 600));
        let rect3 = DeviceIntRect::new(Point2D::new(300, 200), Point2D::new(600, 500));
        assert_eq!(
            coordinates.flip_rect(&rect1),
            DeviceIntRect::new(Point2D::new(0, 200), Point2D::new(700, 600))
        );
        assert_eq!(
            coordinates.flip_rect(&rect2),
            DeviceIntRect::new(Point2D::new(100, 0), Point2D::new(800, 500))
        );
        assert_eq!(
            coordinates.flip_rect(&rect3),
            DeviceIntRect::new(Point2D::new(300, 100), Point2D::new(600, 400))
        );
    }
}
