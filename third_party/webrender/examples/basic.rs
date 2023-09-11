/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate euclid;
extern crate gleam;
extern crate glutin;
extern crate webrender;
extern crate winit;

#[path = "common/boilerplate.rs"]
mod boilerplate;

use crate::boilerplate::{Example, HandyDandyRectBuilder};
use euclid::vec2;
use winit::TouchPhase;
use std::collections::HashMap;
use webrender::ShaderPrecacheFlags;
use webrender::api::*;
use webrender::render_api::*;
use webrender::api::units::*;


#[derive(Debug)]
enum Gesture {
    None,
    Pan,
    Zoom,
}

#[derive(Debug)]
struct Touch {
    id: u64,
    start_x: f32,
    start_y: f32,
    current_x: f32,
    current_y: f32,
}

fn dist(x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let dx = x0 - x1;
    let dy = y0 - y1;
    ((dx * dx) + (dy * dy)).sqrt()
}

impl Touch {
    fn distance_from_start(&self) -> f32 {
        dist(self.start_x, self.start_y, self.current_x, self.current_y)
    }

    fn initial_distance_from_other(&self, other: &Touch) -> f32 {
        dist(self.start_x, self.start_y, other.start_x, other.start_y)
    }

    fn current_distance_from_other(&self, other: &Touch) -> f32 {
        dist(
            self.current_x,
            self.current_y,
            other.current_x,
            other.current_y,
        )
    }
}

struct TouchState {
    active_touches: HashMap<u64, Touch>,
    current_gesture: Gesture,
    start_zoom: f32,
    current_zoom: f32,
    start_pan: DeviceIntPoint,
    current_pan: DeviceIntPoint,
}

enum TouchResult {
    None,
    Pan(DeviceIntPoint),
    Zoom(f32),
}

impl TouchState {
    fn new() -> TouchState {
        TouchState {
            active_touches: HashMap::new(),
            current_gesture: Gesture::None,
            start_zoom: 1.0,
            current_zoom: 1.0,
            start_pan: DeviceIntPoint::zero(),
            current_pan: DeviceIntPoint::zero(),
        }
    }

    fn handle_event(&mut self, touch: winit::Touch) -> TouchResult {
        match touch.phase {
            TouchPhase::Started => {
                debug_assert!(!self.active_touches.contains_key(&touch.id));
                self.active_touches.insert(
                    touch.id,
                    Touch {
                        id: touch.id,
                        start_x: touch.location.x as f32,
                        start_y: touch.location.y as f32,
                        current_x: touch.location.x as f32,
                        current_y: touch.location.y as f32,
                    },
                );
                self.current_gesture = Gesture::None;
            }
            TouchPhase::Moved => {
                match self.active_touches.get_mut(&touch.id) {
                    Some(active_touch) => {
                        active_touch.current_x = touch.location.x as f32;
                        active_touch.current_y = touch.location.y as f32;
                    }
                    None => panic!("move touch event with unknown touch id!"),
                }

                match self.current_gesture {
                    Gesture::None => {
                        let mut over_threshold_count = 0;
                        let active_touch_count = self.active_touches.len();

                        for (_, touch) in &self.active_touches {
                            if touch.distance_from_start() > 8.0 {
                                over_threshold_count += 1;
                            }
                        }

                        if active_touch_count == over_threshold_count {
                            if active_touch_count == 1 {
                                self.start_pan = self.current_pan;
                                self.current_gesture = Gesture::Pan;
                            } else if active_touch_count == 2 {
                                self.start_zoom = self.current_zoom;
                                self.current_gesture = Gesture::Zoom;
                            }
                        }
                    }
                    Gesture::Pan => {
                        let keys: Vec<u64> = self.active_touches.keys().cloned().collect();
                        debug_assert!(keys.len() == 1);
                        let active_touch = &self.active_touches[&keys[0]];
                        let x = active_touch.current_x - active_touch.start_x;
                        let y = active_touch.current_y - active_touch.start_y;
                        self.current_pan.x = self.start_pan.x + x.round() as i32;
                        self.current_pan.y = self.start_pan.y + y.round() as i32;
                        return TouchResult::Pan(self.current_pan);
                    }
                    Gesture::Zoom => {
                        let keys: Vec<u64> = self.active_touches.keys().cloned().collect();
                        debug_assert!(keys.len() == 2);
                        let touch0 = &self.active_touches[&keys[0]];
                        let touch1 = &self.active_touches[&keys[1]];
                        let initial_distance = touch0.initial_distance_from_other(touch1);
                        let current_distance = touch0.current_distance_from_other(touch1);
                        self.current_zoom = self.start_zoom * current_distance / initial_distance;
                        return TouchResult::Zoom(self.current_zoom);
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.active_touches.remove(&touch.id).unwrap();
                self.current_gesture = Gesture::None;
            }
        }

        TouchResult::None
    }
}

fn main() {
    let mut app = App {
        touch_state: TouchState::new(),
    };
    boilerplate::main_wrapper(&mut app, None);
}

struct App {
    touch_state: TouchState,
}

impl Example for App {
    // Make this the only example to test all shaders for compile errors.
    const PRECACHE_SHADER_FLAGS: ShaderPrecacheFlags = ShaderPrecacheFlags::FULL_COMPILE;

    fn render(
        &mut self,
        api: &mut RenderApi,
        builder: &mut DisplayListBuilder,
        txn: &mut Transaction,
        _: DeviceIntSize,
        pipeline_id: PipelineId,
        _document_id: DocumentId,
    ) {
        let content_bounds = LayoutRect::new(LayoutPoint::zero(), LayoutSize::new(800.0, 600.0));
        let root_space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);
        let spatial_id = root_space_and_clip.spatial_id;

        builder.push_simple_stacking_context(
            content_bounds.origin,
            spatial_id,
            PrimitiveFlags::IS_BACKFACE_VISIBLE,
        );

        let image_mask_key = api.generate_image_key();
        txn.add_image(
            image_mask_key,
            ImageDescriptor::new(2, 2, ImageFormat::R8, ImageDescriptorFlags::IS_OPAQUE),
            ImageData::new(vec![0, 80, 180, 255]),
            None,
        );
        let mask = ImageMask {
            image: image_mask_key,
            rect: (75, 75).by(100, 100),
            repeat: false,
        };
        let complex = ComplexClipRegion::new(
            (50, 50).to(150, 150),
            BorderRadius::uniform(20.0),
            ClipMode::Clip
        );
        let mask_clip_id = builder.define_clip_image_mask(
            &root_space_and_clip,
            mask,
            &vec![],
            FillRule::Nonzero,
        );
        let clip_id = builder.define_clip_rounded_rect(
            &SpaceAndClipInfo {
                spatial_id: root_space_and_clip.spatial_id,
                clip_id: mask_clip_id,
            },
            complex,
        );

        builder.push_rect(
            &CommonItemProperties::new(
                (100, 100).to(200, 200),
                SpaceAndClipInfo { spatial_id, clip_id },
            ),
            (100, 100).to(200, 200),
            ColorF::new(0.0, 1.0, 0.0, 1.0),
        );

        builder.push_rect(
            &CommonItemProperties::new(
                (250, 100).to(350, 200),
                SpaceAndClipInfo { spatial_id, clip_id },
            ),
            (250, 100).to(350, 200),
            ColorF::new(0.0, 1.0, 0.0, 1.0),
        );
        let border_side = BorderSide {
            color: ColorF::new(0.0, 0.0, 1.0, 1.0),
            style: BorderStyle::Groove,
        };
        let border_widths = LayoutSideOffsets::new_all_same(10.0);
        let border_details = BorderDetails::Normal(NormalBorder {
            top: border_side,
            right: border_side,
            bottom: border_side,
            left: border_side,
            radius: BorderRadius::uniform(20.0),
            do_aa: true,
        });

        let bounds = (100, 100).to(200, 200);
        builder.push_border(
            &CommonItemProperties::new(
                bounds,
                SpaceAndClipInfo { spatial_id, clip_id },
            ),
            bounds,
            border_widths,
            border_details,
        );

        if false {
            // draw box shadow?
            let simple_box_bounds = (20, 200).by(50, 50);
            let offset = vec2(10.0, 10.0);
            let color = ColorF::new(1.0, 1.0, 1.0, 1.0);
            let blur_radius = 0.0;
            let spread_radius = 0.0;
            let simple_border_radius = 8.0;
            let box_shadow_type = BoxShadowClipMode::Inset;

            builder.push_box_shadow(
                &CommonItemProperties::new(content_bounds, root_space_and_clip),
                simple_box_bounds,
                offset,
                color,
                blur_radius,
                spread_radius,
                BorderRadius::uniform(simple_border_radius),
                box_shadow_type,
            );
        }

        builder.pop_stacking_context();
    }

    fn on_event(&mut self, event: winit::WindowEvent, api: &mut RenderApi, document_id: DocumentId) -> bool {
        let mut txn = Transaction::new();
        match event {
            winit::WindowEvent::Touch(touch) => match self.touch_state.handle_event(touch) {
                TouchResult::Pan(pan) => {
                    txn.set_pan(pan);
                }
                TouchResult::Zoom(zoom) => {
                    txn.set_pinch_zoom(ZoomFactor::new(zoom));
                }
                TouchResult::None => {}
            },
            _ => (),
        }

        if !txn.is_empty() {
            txn.generate_frame(0);
            api.send_transaction(document_id, txn);
        }

        false
    }
}
