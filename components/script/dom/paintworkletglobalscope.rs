/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::PaintWorkletGlobalScopeBinding;
use dom::bindings::codegen::Bindings::PaintWorkletGlobalScopeBinding::PaintWorkletGlobalScopeMethods;
use dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::workletglobalscope::WorkletGlobalScope;
use dom::workletglobalscope::WorkletGlobalScopeInit;
use dom_struct::dom_struct;
use euclid::Size2D;
use ipc_channel::ipc::IpcSharedMemory;
use js::rust::Runtime;
use msg::constellation_msg::PipelineId;
use net_traits::image::base::Image;
use net_traits::image::base::PixelFormat;
use script_traits::PaintWorkletError;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::rc::Rc;
use std::sync::mpsc::Sender;

#[dom_struct]
/// https://drafts.css-houdini.org/css-paint-api/#paintworkletglobalscope
pub struct PaintWorkletGlobalScope {
    /// The worklet global for this object
    worklet_global: WorkletGlobalScope,
    /// A buffer to draw into
    buffer: DOMRefCell<Vec<u8>>,
}

impl PaintWorkletGlobalScope {
    #[allow(unsafe_code)]
    pub fn new(runtime: &Runtime,
               pipeline_id: PipelineId,
               base_url: ServoUrl,
               init: &WorkletGlobalScopeInit)
               -> Root<PaintWorkletGlobalScope> {
        debug!("Creating paint worklet global scope for pipeline {}.", pipeline_id);
        let global = box PaintWorkletGlobalScope {
            worklet_global: WorkletGlobalScope::new_inherited(pipeline_id, base_url, init),
            buffer: Default::default(),
        };
        unsafe { PaintWorkletGlobalScopeBinding::Wrap(runtime.cx(), global) }
    }

    pub fn perform_a_worklet_task(&self, task: PaintWorkletTask) {
        match task {
            PaintWorkletTask::DrawAPaintImage(name, size, sender) => self.draw_a_paint_image(name, size, sender),
        }
    }

    fn draw_a_paint_image(&self,
                          name: Atom,
                          concrete_object_size: Size2D<Au>,
                          sender: Sender<Result<Image, PaintWorkletError>>) {
        let width = concrete_object_size.width.to_px().abs() as u32;
        let height = concrete_object_size.height.to_px().abs() as u32;
        let area = (width as usize) * (height as usize);
        let old_buffer_size = self.buffer.borrow().len();
        let new_buffer_size = area * 4;
        debug!("Drawing a paint image {}({},{}).", name, width, height);
        // TODO: call into script to create the image.
        // For now, we just build a dummy.
        if new_buffer_size > old_buffer_size {
            let pixel = [0xFF, 0x00, 0x00, 0xFF];
            self.buffer.borrow_mut().extend(pixel.iter().cycle().take(new_buffer_size - old_buffer_size));
        } else {
            self.buffer.borrow_mut().truncate(new_buffer_size);
        }
        let image = Image {
            width: width,
            height: height,
            format: PixelFormat::BGRA8,
            bytes: IpcSharedMemory::from_bytes(&*self.buffer.borrow()),
            id: None,
        };
        let _ = sender.send(Ok(image));
    }
}

impl PaintWorkletGlobalScopeMethods for PaintWorkletGlobalScope {
    /// https://drafts.css-houdini.org/css-paint-api/#dom-paintworkletglobalscope-registerpaint
    fn RegisterPaint(&self, name: DOMString, _paintCtor: Rc<VoidFunction>) {
        debug!("Registering paint image name {}.", name);
        // TODO
    }
}

/// Tasks which can be peformed by a paint worklet
pub enum PaintWorkletTask {
    DrawAPaintImage(Atom, Size2D<Au>, Sender<Result<Image, PaintWorkletError>>)
}
