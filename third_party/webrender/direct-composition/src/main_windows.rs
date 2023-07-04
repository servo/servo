/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate direct_composition;
extern crate euclid;
extern crate gleam;
extern crate webrender;
extern crate winit;

use euclid::size2;
use direct_composition::DirectComposition;
use std::sync::mpsc;
use webrender::api;
use winit::os::windows::{WindowExt, WindowBuilderExt};
use winit::dpi::LogicalSize;

fn main() {
    let mut events_loop = winit::EventsLoop::new();

    let (tx, rx) = mpsc::channel();
    let notifier = Box::new(Notifier { events_proxy: events_loop.create_proxy(), tx });

    let window = winit::WindowBuilder::new()
        .with_title("WebRender + ANGLE + DirectComposition")
        .with_dimensions(LogicalSize::new(1024., 768.))
        .with_decorations(true)
        .with_transparency(true)
        .with_no_redirection_bitmap(true)
        .build(&events_loop)
        .unwrap();

    let composition = direct_composition_from_window(&window);
    let factor = window.get_hidpi_factor() as f32;

    let mut clicks: usize = 0;
    let mut offset_y = 100.;
    let mut rects = [
        Rectangle::new(&composition, &notifier, factor, size2(300, 200), 0., 0.2, 0.4, 1.),
        Rectangle::new(&composition, &notifier, factor, size2(400, 300), 0., 0.5, 0., 0.5),
    ];
    rects[0].render(factor, &rx);
    rects[1].render(factor, &rx);

    rects[0].visual.set_offset_x(100.);
    rects[0].visual.set_offset_y(50.);

    rects[1].visual.set_offset_x(200.);
    rects[1].visual.set_offset_y(offset_y);

    composition.commit();

    events_loop.run_forever(|event| {
        if let winit::Event::WindowEvent { event, .. } = event {
            match event {
                winit::WindowEvent::CloseRequested => {
                    return winit::ControlFlow::Break
                }
                winit::WindowEvent::MouseWheel { delta, .. } => {
                    let dy = match delta {
                        winit::MouseScrollDelta::LineDelta(_, dy) => dy,
                        winit::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };
                    offset_y = (offset_y - 10. * dy).max(0.).min(468.);

                    rects[1].visual.set_offset_y(offset_y);
                    composition.commit();
                }
                winit::WindowEvent::MouseInput {
                    button: winit::MouseButton::Left,
                    state: winit::ElementState::Pressed,
                    ..
                } => {
                    clicks += 1;
                    let rect = &mut rects[clicks % 2];
                    rect.color.g += 0.1;
                    rect.color.g %= 1.;
                    rect.render(factor, &rx)
                }
                _ => {}
            }
        }
        winit::ControlFlow::Continue
    });
}

fn direct_composition_from_window(window: &winit::Window) -> DirectComposition {
    unsafe {
        DirectComposition::new(window.get_hwnd() as _)
    }
}

struct Rectangle {
    visual: direct_composition::AngleVisual,
    renderer: Option<webrender::Renderer>,
    api: api::RenderApi,
    document_id: api::DocumentId,
    size: api::units::DeviceIntSize,
    color: api::ColorF,
}

impl Rectangle {
    fn new(composition: &DirectComposition, notifier: &Box<Notifier>,
           device_pixel_ratio: f32, size: api::units::DeviceIntSize, r: f32, g: f32, b: f32, a: f32)
           -> Self {
        let visual = composition.create_angle_visual(size.width as u32, size.height as u32);
        visual.make_current();

        let (renderer, sender) = webrender::Renderer::new(
            composition.gleam.clone(),
            notifier.clone(),
            webrender::RendererOptions {
                clear_color: Some(api::ColorF::new(0., 0., 0., 0.)),
                device_pixel_ratio,
                ..webrender::RendererOptions::default()
            },
            None,
            size,
        ).unwrap();
        let api = sender.create_api();

       Rectangle {
            visual,
            renderer: Some(renderer),
            document_id: api.add_document(size, 0),
            api,
            size,
            color: api::ColorF { r, g, b, a },
        }
    }

    fn render(&mut self, device_pixel_ratio: f32, rx: &mpsc::Receiver<()>) {
        self.visual.make_current();

        let pipeline_id = api::PipelineId(0, 0);
        let layout_size = self.size.to_f32() / euclid::Scale::new(device_pixel_ratio);
        let mut builder = api::DisplayListBuilder::new(pipeline_id, layout_size);

        let rect = euclid::Rect::new(euclid::Point2D::zero(), layout_size);

        let region = api::ComplexClipRegion::new(
            rect,
            api::BorderRadius::uniform(20.),
            api::ClipMode::Clip
        );
        let clip_id = builder.define_clip_rounded_rect(
            &api::SpaceAndClipInfo::root_scroll(pipeline_id),
            region,
        );

        builder.push_rect(
            &api::CommonItemProperties::new(
                rect,
                api::SpaceAndClipInfo {
                    spatial_id: api::SpatialId::root_scroll_node(pipeline_id),
                    clip_id,
                },
            ),
            rect,
            self.color,
        );

        let mut transaction = api::Transaction::new();
        transaction.set_display_list(
            api::Epoch(0),
            None,
            layout_size,
            builder.finalize(),
            true,
        );
        transaction.set_root_pipeline(pipeline_id);
        transaction.generate_frame();
        self.api.send_transaction(self.document_id, transaction);
        rx.recv().unwrap();
        let renderer = self.renderer.as_mut().unwrap();
        renderer.update();
        renderer.render(self.size).unwrap();
        let _ = renderer.flush_pipeline_info();
        self.visual.present();
    }
}

impl Drop for Rectangle {
    fn drop(&mut self) {
        self.renderer.take().unwrap().deinit()
    }
}

#[derive(Clone)]
struct Notifier {
    events_proxy: winit::EventsLoopProxy,
    tx: mpsc::Sender<()>,
}

impl api::RenderNotifier for Notifier {
    fn clone(&self) -> Box<dyn api::RenderNotifier> {
        Box::new(Clone::clone(self))
    }

    fn wake_up(&self) {
        self.tx.send(()).unwrap();
        let _ = self.events_proxy.wakeup();
    }

    fn new_frame_ready(&self,
                       _: api::DocumentId,
                       _: bool,
                       _: bool,
                       _: Option<u64>) {
        self.wake_up();
    }
}
