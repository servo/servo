/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::{
    Canvas2dMsg, CanvasId, CanvasMsg, FillOrStrokeStyle, RepetitionStyle, SurfaceId, SurfaceStyle,
};
use dom_struct::dom_struct;
use euclid::default::{Size2D, Transform2D};
use ipc_channel::ipc::IpcSender;
use pixels::IpcSnapshot;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasPatternMethods;
use crate::dom::bindings::codegen::Bindings::DOMMatrixBinding::DOMMatrix2DInit;
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::canvasgradient::ToFillOrStrokeStyle;
use crate::dom::dommatrixreadonly::dommatrix2dinit_to_matrix;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

/// <https://html.spec.whatwg.org/multipage/#canvaspattern>
#[dom_struct]
pub(crate) struct CanvasPattern {
    reflector_: Reflector,
    #[no_trace]
    canvas_id: CanvasId,
    #[no_trace]
    surface_id: SurfaceId,
    #[no_trace]
    surface_size: Size2D<u32>,
    repeat_x: bool,
    repeat_y: bool,
    #[no_trace]
    transform: DomRefCell<Transform2D<f32>>,
    origin_clean: bool,
    #[no_trace]
    sender: IpcSender<CanvasMsg>,
}

impl CanvasPattern {
    fn new_inherited(
        canvas_id: CanvasId,
        surface_id: SurfaceId,
        surface_size: Size2D<u32>,
        repeat: RepetitionStyle,
        origin_clean: bool,
        sender: IpcSender<CanvasMsg>,
    ) -> CanvasPattern {
        let (x, y) = match repeat {
            RepetitionStyle::Repeat => (true, true),
            RepetitionStyle::RepeatX => (true, false),
            RepetitionStyle::RepeatY => (false, true),
            RepetitionStyle::NoRepeat => (false, false),
        };

        CanvasPattern {
            reflector_: Reflector::new(),
            canvas_id,
            surface_id,
            surface_size,
            repeat_x: x,
            repeat_y: y,
            transform: DomRefCell::new(Transform2D::identity()),
            origin_clean,
            sender,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        canvas_id: CanvasId,
        surface_id: SurfaceId,
        snapshot: IpcSnapshot,
        repeat: RepetitionStyle,
        origin_clean: bool,
        sender: IpcSender<CanvasMsg>,
        can_gc: CanGc,
    ) -> DomRoot<CanvasPattern> {
        let size = snapshot.size();

        if let Err(e) = sender.send(CanvasMsg::Canvas2d(
            Canvas2dMsg::CreateSurfacePattern(surface_id, snapshot),
            canvas_id,
        )) {
            error!("Error sending CreateSurfacePattern: {e:?}");
        }

        reflect_dom_object(
            Box::new(CanvasPattern::new_inherited(
                canvas_id,
                surface_id,
                size,
                repeat,
                origin_clean,
                sender,
            )),
            global,
            can_gc,
        )
    }
    pub(crate) fn origin_is_clean(&self) -> bool {
        self.origin_clean
    }
}

impl CanvasPatternMethods<crate::DomTypeHolder> for CanvasPattern {
    /// <https://html.spec.whatwg.org/multipage/#dom-canvaspattern-settransform>
    fn SetTransform(&self, transform: &DOMMatrix2DInit) -> ErrorResult {
        // Step 1. Let matrix be the result of creating a DOMMatrix from the 2D
        // dictionary transform.
        let matrix = dommatrix2dinit_to_matrix(transform)?;

        // Step 2. If one or more of matrix's m11 element, m12 element, m21
        // element, m22 element, m41 element, or m42 element are infinite or
        // NaN, then return.
        if !matrix.m11.is_finite() ||
            !matrix.m12.is_finite() ||
            !matrix.m21.is_finite() ||
            !matrix.m22.is_finite() ||
            !matrix.m31.is_finite() ||
            !matrix.m32.is_finite()
        {
            return Ok(());
        }

        // Step 3. Reset the pattern's transformation matrix to matrix.
        *self.transform.borrow_mut() = matrix.cast();

        Ok(())
    }
}

impl Drop for CanvasPattern {
    // This is OK because we only access non JS managed fields
    fn drop(&mut self) {
        if let Err(e) = self.sender.send(CanvasMsg::Canvas2d(
            Canvas2dMsg::DropSurfacePattern(self.surface_id),
            self.canvas_id,
        )) {
            error!("Error sending DropSurfacePattern: {e:?}");
        }
    }
}

impl ToFillOrStrokeStyle for &CanvasPattern {
    fn to_fill_or_stroke_style(self) -> FillOrStrokeStyle {
        FillOrStrokeStyle::Surface(SurfaceStyle::new(
            self.surface_id,
            self.surface_size,
            self.repeat_x,
            self.repeat_y,
            *self.transform.borrow(),
        ))
    }
}
