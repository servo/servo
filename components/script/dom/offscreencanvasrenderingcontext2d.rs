/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding::OffscreenCanvasRenderingContext2DMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom,DomRoot};
use dom::globalscope::GlobalScope;
use dom::offscreencanvas::{OffscreenCanvas};
use dom_struct::dom_struct;
use euclid::Size2D;

#[dom_struct]
pub struct OffscreenCanvasRenderingContext2D{
    reflector_: Reflector,
    canvas: Option<Dom<OffscreenCanvas>>,
}

impl OffscreenCanvasRenderingContext2D {
    pub fn new_inherited(
        global: &GlobalScope,
        canvas: Option<&OffscreenCanvas>,
        size: Size2D<u64>,
    ) -> OffscreenCanvasRenderingContext2D {
        OffscreenCanvasRenderingContext2D {
            reflector_: Reflector::new(),
            canvas: canvas.map(Dom::from_ref),
        }
    }

    pub fn new(
        global: &GlobalScope,
        canvas: &OffscreenCanvas,
        size: Size2D<u64>,
    ) -> DomRoot<OffscreenCanvasRenderingContext2D> {
        //let window = window_from_node(canvas);
        let boxed = Box::new(OffscreenCanvasRenderingContext2D::new_inherited(
            global,
            Some(canvas),
            size,
        ));
        reflect_dom_object(boxed, global, OffscreenCanvasRenderingContext2DBinding::Wrap)
    }
}

impl OffscreenCanvasRenderingContext2DMethods for OffscreenCanvasRenderingContext2D {

    fn Canvas(&self) -> DomRoot<OffscreenCanvas> {
        DomRoot::from_ref(self.canvas.as_ref().expect("No canvas."))
    }

    // ***************** For Mixin CanvasState ***********************************
	// https://html.spec.whatwg.org/multipage/#dom-context-2d-save
	fn Save(&self) {
		self.saved_states
			.borrow_mut()
			.push(self.state.borrow().clone());
		self.send_canvas_2d_msg(Canvas2dMsg::SaveContext);
	}

	#[allow(unrooted_must_root)]
	// https://html.spec.whatwg.org/multipage/#dom-context-2d-restore
	fn Restore(&self) {
		let mut saved_states = self.saved_states.borrow_mut();
		if let Some(state) = saved_states.pop() {
			self.state.borrow_mut().clone_from(&state);
			self.send_canvas_2d_msg(Canvas2dMsg::RestoreContext);
		}
	}

	// ******** for mixin CanvasTransform ***************************************

	// https://html.spec.whatwg.org/multipage/#dom-context-2d-scale
	fn Scale(&self, x: f64, y: f64) {
		if !(x.is_finite() && y.is_finite()) {
			return;
		}

		let transform = self.state.borrow().transform;
		self.state.borrow_mut().transform = transform.pre_scale(x as f32, y as f32);
		self.update_transform()
	}

	// https://html.spec.whatwg.org/multipage/#dom-context-2d-rotate
	fn Rotate(&self, angle: f64) {
		if angle == 0.0 || !angle.is_finite() {
			return;
		}

		let (sin, cos) = (angle.sin(), angle.cos());
		let transform = self.state.borrow().transform;
		self.state.borrow_mut().transform = transform.pre_mul(&Transform2D::row_major(
			cos as f32,
			sin as f32,
			-sin as f32,
			cos as f32,
			0.0,
			0.0,
		));
		self.update_transform()
	}

	// https://html.spec.whatwg.org/multipage/#dom-context-2d-translate
	fn Translate(&self, x: f64, y: f64) {
		if !(x.is_finite() && y.is_finite()) {
			return;
		}

		let transform = self.state.borrow().transform;
		self.state.borrow_mut().transform = transform.pre_translate(vec2(x as f32, y as f32));
		self.update_transform()
	}

	// https://html.spec.whatwg.org/multipage/#dom-context-2d-transform
	fn Transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
		if !(a.is_finite() &&
			b.is_finite() &&
			c.is_finite() &&
			d.is_finite() &&
			e.is_finite() &&
			f.is_finite())
		{
			return;
		}

		let transform = self.state.borrow().transform;
		self.state.borrow_mut().transform = transform.pre_mul(&Transform2D::row_major(
			a as f32, b as f32, c as f32, d as f32, e as f32, f as f32,
		));
		self.update_transform()
	}

	// https://html.spec.whatwg.org/multipage/#dom-context-2d-settransform
	fn SetTransform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
		if !(a.is_finite() &&
			b.is_finite() &&
			c.is_finite() &&
			d.is_finite() &&
			e.is_finite() &&
			f.is_finite())
		{
			return;
		}

		self.state.borrow_mut().transform =
			Transform2D::row_major(a as f32, b as f32, c as f32, d as f32, e as f32, f as f32);
		self.update_transform()
	}

	// https://html.spec.whatwg.org/multipage/#dom-context-2d-resettransform
	fn ResetTransform(&self) {
		self.state.borrow_mut().transform = Transform2D::identity();
		self.update_transform()
	}



	 fn GlobalAlpha(&self) -> f64 {
		 let state = self.state.borrow();
		 state.global_alpha
	 }

	 // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
	 fn SetGlobalAlpha(&self, alpha: f64) {
		 if !alpha.is_finite() || alpha > 1.0 || alpha < 0.0 {
			 return;
		 }

		 self.state.borrow_mut().global_alpha = alpha;
		 self.send_canvas_2d_msg(Canvas2dMsg::SetGlobalAlpha(alpha as f32))
	 }

	 // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
	 fn GlobalCompositeOperation(&self) -> DOMString {
		 let state = self.state.borrow();
		 match state.global_composition {
			 CompositionOrBlending::Composition(op) => DOMString::from(op.to_str()),
			 CompositionOrBlending::Blending(op) => DOMString::from(op.to_str()),
		 }
	 }

	 // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
	 fn SetGlobalCompositeOperation(&self, op_str: DOMString) {
		 if let Ok(op) = CompositionOrBlending::from_str(&op_str) {
			 self.state.borrow_mut().global_composition = op;
			 self.send_canvas_2d_msg(Canvas2dMsg::SetGlobalComposition(op))
		 }
	 }




	 fn CreateImageData(&self, sw: i32, sh: i32) -> Fallible<DomRoot<ImageData>> {
		 if sw == 0 || sh == 0 {
			 return Err(Error::IndexSize);
		 }
		 ImageData::new(&self.global(), sw.abs() as u32, sh.abs() as u32, None)
	 }

	 // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
	 fn CreateImageData_(&self, imagedata: &ImageData) -> Fallible<DomRoot<ImageData>> {
		 ImageData::new(&self.global(), imagedata.Width(), imagedata.Height(), None)
	 }

	 // https://html.spec.whatwg.org/multipage/#dom-context-2d-getimagedata
	 fn GetImageData(&self, sx: i32, sy: i32, sw: i32, sh: i32) -> Fallible<DomRoot<ImageData>> {
		 // FIXME(nox): There are many arithmetic operations here that can
		 // overflow or underflow, this should probably be audited.

		 if sw == 0 || sh == 0 {
			 return Err(Error::IndexSize);
		 }

		 if !self.origin_is_clean() {
			 return Err(Error::Security);
		 }

		 let (origin, size) = adjust_size_sign(Point2D::new(sx, sy), Size2D::new(sw, sh));
		 // FIXME(nox): This is probably wrong when this is a context for an
		 // offscreen canvas.
		 let canvas_size = self.canvas.as_ref().map_or(Size2D::zero(), |c| c.get_size());
		 let read_rect = match pixels::clip(origin, size, canvas_size) {
			 Some(rect) => rect,
			 None => {
				 // All the pixels are outside the canvas surface.
				 return ImageData::new(&self.global(), size.width, size.height, None);
			 },
		 };

		 ImageData::new(&self.global(), size.width, size.height, Some(self.get_rect(read_rect)))
	 }

	 // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
	 fn PutImageData(&self, imagedata: &ImageData, dx: i32, dy: i32) {
		 self.PutImageData_(imagedata, dx, dy, 0, 0, imagedata.Width() as i32, imagedata.Height() as i32)
	 }

	 // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
	 #[allow(unsafe_code)]
	 fn PutImageData_(
		 &self,
		 imagedata: &ImageData,
		 dx: i32,
		 dy: i32,
		 dirty_x: i32,
		 dirty_y: i32,
		 dirty_width: i32,
		 dirty_height: i32,
	 ) {
		 // FIXME(nox): There are many arithmetic operations here that can
		 // overflow or underflow, this should probably be audited.


		 let imagedata_size = Size2D::new(imagedata.Width(), imagedata.Height());
		 if imagedata_size.area() == 0 {
			 return;
		 }

		 // Step 1.
		 // Done later.

		 // Step 2.
		 // TODO: throw InvalidState if buffer is detached.

		 // FIXME(nox): This is probably wrong when this is a context for an
		 // offscreen canvas.
		 let canvas_size = self.canvas.as_ref().map_or(Size2D::zero(), |c| c.get_size());

		 // Steps 3-6.
		 let (src_origin, src_size) = adjust_size_sign(
			 Point2D::new(dirty_x, dirty_y),
			 Size2D::new(dirty_width, dirty_height),
		 );
		 let src_rect = match pixels::clip(src_origin, src_size, imagedata_size) {
			 Some(rect) => rect,
			 None => return,
		 };
		 let (dst_origin, _) = adjust_size_sign(
			 Point2D::new(dirty_x.saturating_add(dx), dirty_y.saturating_add(dy)),
			 Size2D::new(dirty_width, dirty_height),
		 );
		 // By clipping to the canvas surface, we avoid sending any pixel
		 // that would fall outside it.
		 let dst_rect = match pixels::clip(dst_origin, src_rect.size, canvas_size) {
			 Some(rect) => rect,
			 None => return,
		 };

		 // Step 7.
		 let (sender, receiver) = ipc::bytes_channel().unwrap();
		 let pixels = unsafe {
			 &imagedata.get_rect(Rect::new(src_rect.origin, dst_rect.size))
		 };
		 self.send_canvas_2d_msg(Canvas2dMsg::PutImageData(dst_rect, receiver));
		 sender.send(pixels).unwrap();
		 self.mark_as_dirty();
	 }

	 pub trait LayoutCanvasRenderingContext2DHelpers {
		#[allow(unsafe_code)]
		unsafe fn GetImageData(&self) -> IpcSender<CanvasMsg>;
		#[allow(unsafe_code)]
		unsafe fn PutImageData_(&self) -> CanvasId;
	}
}
