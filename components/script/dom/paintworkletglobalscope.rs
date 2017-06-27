/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::CanvasData;
use dom::bindings::callback::CallbackContainer;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::PaintWorkletGlobalScopeBinding;
use dom::bindings::codegen::Bindings::PaintWorkletGlobalScopeBinding::PaintWorkletGlobalScopeMethods;
use dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use dom::bindings::conversions::get_property;
use dom::bindings::conversions::get_property_jsval;
use dom::bindings::error::Error;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::JS;
use dom::bindings::js::Root;
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::cssstylevalue::CSSStyleValue;
use dom::paintrenderingcontext2d::PaintRenderingContext2D;
use dom::paintsize::PaintSize;
use dom::stylepropertymapreadonly::StylePropertyMapReadOnly;
use dom::worklet::WorkletExecutor;
use dom::workletglobalscope::WorkletGlobalScope;
use dom::workletglobalscope::WorkletGlobalScopeInit;
use dom::workletglobalscope::WorkletTask;
use dom_struct::dom_struct;
use euclid::ScaleFactor;
use euclid::TypedSize2D;
use ipc_channel::ipc;
use js::jsapi::Call;
use js::jsapi::Construct1;
use js::jsapi::HandleValue;
use js::jsapi::HandleValueArray;
use js::jsapi::Heap;
use js::jsapi::IsCallable;
use js::jsapi::IsConstructor;
use js::jsapi::JSAutoCompartment;
use js::jsapi::JS_ClearPendingException;
use js::jsapi::JS_IsExceptionPending;
use js::jsapi::JS_NewArrayObject;
use js::jsval::JSVal;
use js::jsval::ObjectValue;
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use msg::constellation_msg::PipelineId;
use net_traits::image::base::PixelFormat;
use net_traits::image_cache::ImageCache;
use script_layout_interface::message::Msg;
use script_traits::DrawAPaintImageResult;
use script_traits::Painter;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use style_traits::CSSPixel;
use style_traits::DevicePixel;

/// https://drafts.css-houdini.org/css-paint-api/#paintworkletglobalscope
#[dom_struct]
pub struct PaintWorkletGlobalScope {
    /// The worklet global for this object
    worklet_global: WorkletGlobalScope,
    /// The image cache
    #[ignore_heap_size_of = "Arc"]
    image_cache: Arc<ImageCache>,
    /// https://drafts.css-houdini.org/css-paint-api/#paint-definitions
    paint_definitions: DOMRefCell<HashMap<Atom, Box<PaintDefinition>>>,
    /// https://drafts.css-houdini.org/css-paint-api/#paint-class-instances
    paint_class_instances: DOMRefCell<HashMap<Atom, Box<Heap<JSVal>>>>,
}

impl PaintWorkletGlobalScope {
    #[allow(unsafe_code)]
    pub fn new(runtime: &Runtime,
               pipeline_id: PipelineId,
               base_url: ServoUrl,
               executor: WorkletExecutor,
               init: &WorkletGlobalScopeInit)
               -> Root<PaintWorkletGlobalScope> {
        debug!("Creating paint worklet global scope for pipeline {}.", pipeline_id);
        let global = box PaintWorkletGlobalScope {
            worklet_global: WorkletGlobalScope::new_inherited(pipeline_id, base_url, executor, init),
            image_cache: init.image_cache.clone(),
            paint_definitions: Default::default(),
            paint_class_instances: Default::default(),
        };
        unsafe { PaintWorkletGlobalScopeBinding::Wrap(runtime.cx(), global) }
    }

    pub fn image_cache(&self) -> Arc<ImageCache> {
        self.image_cache.clone()
    }

    pub fn perform_a_worklet_task(&self, task: PaintWorkletTask) {
        match task {
            PaintWorkletTask::DrawAPaintImage(name, size_in_px, device_pixel_ratio, properties, arguments, sender) => {
                let properties = StylePropertyMapReadOnly::from_iter(self.upcast(), properties);
                let result = self.draw_a_paint_image(name, size_in_px, device_pixel_ratio, &*properties, arguments);
                let _ = sender.send(result);
            }
        }
    }

    /// https://drafts.css-houdini.org/css-paint-api/#draw-a-paint-image
    fn draw_a_paint_image(&self,
                          name: Atom,
                          size_in_px: TypedSize2D<f32, CSSPixel>,
                          device_pixel_ratio: ScaleFactor<f32, CSSPixel, DevicePixel>,
                          properties: &StylePropertyMapReadOnly,
                          arguments: Vec<String>)
                          -> DrawAPaintImageResult
    {
        let size_in_dpx = size_in_px * device_pixel_ratio;
        let size_in_dpx = TypedSize2D::new(size_in_dpx.width.abs() as u32, size_in_dpx.height.abs() as u32);

        // TODO: Steps 1-5.

        // TODO: document paint definitions.
        self.invoke_a_paint_callback(name, size_in_px, size_in_dpx, device_pixel_ratio, properties, arguments)
    }

    /// https://drafts.css-houdini.org/css-paint-api/#invoke-a-paint-callback
    #[allow(unsafe_code)]
    fn invoke_a_paint_callback(&self,
                               name: Atom,
                               size_in_px: TypedSize2D<f32, CSSPixel>,
                               size_in_dpx: TypedSize2D<u32, DevicePixel>,
                               device_pixel_ratio: ScaleFactor<f32, CSSPixel, DevicePixel>,
                               properties: &StylePropertyMapReadOnly,
                               mut arguments: Vec<String>)
                               -> DrawAPaintImageResult
    {
        debug!("Invoking a paint callback {}({},{}) at {}.",
               name, size_in_px.width, size_in_px.height, device_pixel_ratio);

        let cx = self.worklet_global.get_cx();
        let _ac = JSAutoCompartment::new(cx, self.worklet_global.reflector().get_jsobject().get());

        // TODO: Steps 1-2.1.
        // Step 2.2-5.1.
        rooted!(in(cx) let mut class_constructor = UndefinedValue());
        rooted!(in(cx) let mut paint_function = UndefinedValue());
        let rendering_context = match self.paint_definitions.borrow().get(&name) {
            None => {
                // Step 2.2.
                warn!("Drawing un-registered paint definition {}.", name);
                return self.invalid_image(size_in_dpx, vec![]);
            }
            Some(definition) => {
                // Step 5.1
                if !definition.constructor_valid_flag.get() {
                    debug!("Drawing invalid paint definition {}.", name);
                    return self.invalid_image(size_in_dpx, vec![]);
                }
                class_constructor.set(definition.class_constructor.get());
                paint_function.set(definition.paint_function.get());
                Root::from_ref(&*definition.context)
            }
        };

        // Steps 5.2-5.4
        // TODO: the spec requires calling the constructor now, but we might want to
        // prepopulate the paint instance in `RegisterPaint`, to avoid calling it in
        // the primary worklet thread.
        // https://github.com/servo/servo/issues/17377
        rooted!(in(cx) let mut paint_instance = UndefinedValue());
        match self.paint_class_instances.borrow_mut().entry(name.clone()) {
            Entry::Occupied(entry) => paint_instance.set(entry.get().get()),
            Entry::Vacant(entry) => {
                // Step 5.2-5.3
                let args = HandleValueArray::new();
                rooted!(in(cx) let mut result = null_mut());
                unsafe { Construct1(cx, class_constructor.handle(), &args, result.handle_mut()); }
                paint_instance.set(ObjectValue(result.get()));
                if unsafe { JS_IsExceptionPending(cx) } {
                    debug!("Paint constructor threw an exception {}.", name);
                    unsafe { JS_ClearPendingException(cx); }
                    self.paint_definitions.borrow_mut().get_mut(&name)
                        .expect("Vanishing paint definition.")
                        .constructor_valid_flag.set(false);
                    return self.invalid_image(size_in_dpx, vec![]);
                }
                // Step 5.4
                entry.insert(Box::new(Heap::default())).set(paint_instance.get());
            }
        };

        // TODO: Steps 6-7
        // Step 8
        // TODO: the spec requires creating a new paint rendering context each time,
        // this code recycles the same one.
        rendering_context.set_bitmap_dimensions(size_in_px, device_pixel_ratio);

        // Step 9
        let paint_size = PaintSize::new(self, size_in_px);

        // TODO: Step 10
        // Steps 11-12
        debug!("Invoking paint function {}.", name);
        rooted_vec!(let arguments_values <- arguments.drain(..)
                    .map(|argument| CSSStyleValue::new(self.upcast(), argument)));
        let arguments_value_vec: Vec<JSVal> = arguments_values.iter()
            .map(|argument| ObjectValue(argument.reflector().get_jsobject().get()))
            .collect();
        let arguments_value_array = unsafe { HandleValueArray::from_rooted_slice(&*arguments_value_vec) };
        rooted!(in(cx) let argument_object = unsafe { JS_NewArrayObject(cx, &arguments_value_array) });

        let args_slice = [
            ObjectValue(rendering_context.reflector().get_jsobject().get()),
            ObjectValue(paint_size.reflector().get_jsobject().get()),
            ObjectValue(properties.reflector().get_jsobject().get()),
            ObjectValue(argument_object.get()),
        ];
        let args = unsafe { HandleValueArray::from_rooted_slice(&args_slice) };

        rooted!(in(cx) let mut result = UndefinedValue());
        unsafe { Call(cx, paint_instance.handle(), paint_function.handle(), &args, result.handle_mut()); }
        let missing_image_urls = rendering_context.take_missing_image_urls();

        // Step 13.
        if unsafe { JS_IsExceptionPending(cx) } {
            debug!("Paint function threw an exception {}.", name);
            unsafe { JS_ClearPendingException(cx); }
            return self.invalid_image(size_in_dpx, missing_image_urls);
        }

        let (sender, receiver) = ipc::channel().expect("IPC channel creation.");
        rendering_context.send_data(sender);
        let image_key = match receiver.recv() {
            Ok(CanvasData::Image(data)) => Some(data.image_key),
            _ => None,
        };

        DrawAPaintImageResult {
            width: size_in_dpx.width,
            height: size_in_dpx.height,
            format: PixelFormat::BGRA8,
            image_key: image_key,
            missing_image_urls: missing_image_urls,
        }
    }

    // https://drafts.csswg.org/css-images-4/#invalid-image
    fn invalid_image(&self, size: TypedSize2D<u32, DevicePixel>, missing_image_urls: Vec<ServoUrl>)
                     -> DrawAPaintImageResult {
        debug!("Returning an invalid image.");
        DrawAPaintImageResult {
            width: size.width as u32,
            height: size.height as u32,
            format: PixelFormat::BGRA8,
            image_key: None,
            missing_image_urls: missing_image_urls,
        }
    }

    fn painter(&self, name: Atom) -> Arc<Painter> {
        // Rather annoyingly we have to use a mutex here to make the painter Sync.
        struct WorkletPainter(Atom, Mutex<WorkletExecutor>);
        impl Painter for WorkletPainter {
            fn draw_a_paint_image(&self,
                                  size: TypedSize2D<f32, CSSPixel>,
                                  device_pixel_ratio: ScaleFactor<f32, CSSPixel, DevicePixel>,
                                  properties: Vec<(Atom, String)>,
                                  arguments: Vec<String>)
                                  -> DrawAPaintImageResult {
                let name = self.0.clone();
                let (sender, receiver) = mpsc::channel();
                let task = PaintWorkletTask::DrawAPaintImage(name,
                                                             size,
                                                             device_pixel_ratio,
                                                             properties,
                                                             arguments,
                                                             sender);
                self.1.lock().expect("Locking a painter.")
                    .schedule_a_worklet_task(WorkletTask::Paint(task));
                receiver.recv().expect("Worklet thread died?")
            }
        }
        Arc::new(WorkletPainter(name, Mutex::new(self.worklet_global.executor())))
    }
}

impl PaintWorkletGlobalScopeMethods for PaintWorkletGlobalScope {
    #[allow(unsafe_code)]
    #[allow(unrooted_must_root)]
    /// https://drafts.css-houdini.org/css-paint-api/#dom-paintworkletglobalscope-registerpaint
    fn RegisterPaint(&self, name: DOMString, paint_ctor: Rc<VoidFunction>) -> Fallible<()> {
        let name = Atom::from(name);
        let cx = self.worklet_global.get_cx();
        rooted!(in(cx) let paint_obj = paint_ctor.callback_holder().get());
        rooted!(in(cx) let paint_val = ObjectValue(paint_obj.get()));

        debug!("Registering paint image name {}.", name);

        // Step 1.
        if name.is_empty() {
            return Err(Error::Type(String::from("Empty paint name."))) ;
        }

        // Step 2-3.
        if self.paint_definitions.borrow().contains_key(&name) {
            return Err(Error::InvalidModification);
        }

        // Step 4-6.
        let mut property_names: Vec<String> =
            unsafe { get_property(cx, paint_obj.handle(), "inputProperties", ()) }?
            .unwrap_or_default();
        let properties = property_names.drain(..).map(Atom::from).collect();

        // Step 7-9.
        let input_arguments: Vec<String> =
            unsafe { get_property(cx, paint_obj.handle(), "inputArguments", ()) }?
            .unwrap_or_default();

        // TODO: Steps 10-11.

        // Steps 12-13.
        let alpha: bool =
            unsafe { get_property(cx, paint_obj.handle(), "alpha", ()) }?
            .unwrap_or(true);

        // Step 14
        if unsafe { !IsConstructor(paint_obj.get()) } {
            return Err(Error::Type(String::from("Not a constructor.")));
        }

        // Steps 15-16
        rooted!(in(cx) let mut prototype = UndefinedValue());
        unsafe { get_property_jsval(cx, paint_obj.handle(), "prototype", prototype.handle_mut())?; }
        if !prototype.is_object() {
            return Err(Error::Type(String::from("Prototype is not an object.")));
        }
        rooted!(in(cx) let prototype = prototype.to_object());

        // Steps 17-18
        rooted!(in(cx) let mut paint_function = UndefinedValue());
        unsafe { get_property_jsval(cx, prototype.handle(), "paint", paint_function.handle_mut())?; }
        if !paint_function.is_object() || unsafe { !IsCallable(paint_function.to_object()) } {
            return Err(Error::Type(String::from("Paint function is not callable.")));
        }

        // Step 19.
        let context = PaintRenderingContext2D::new(self);
        let definition = PaintDefinition::new(paint_val.handle(),
                                              paint_function.handle(),
                                              alpha,
                                              input_arguments.len(),
                                              &*context);

        // Step 20.
        debug!("Registering definition {}.", name);
        self.paint_definitions.borrow_mut().insert(name.clone(), definition);

        // TODO: Step 21.

        // Inform layout that there is a registered paint worklet.
        // TODO: layout will end up getting this message multiple times.
        let painter = self.painter(name.clone());
        let msg = Msg::RegisterPaint(name, properties, painter);
        self.worklet_global.send_to_layout(msg);

        Ok(())
    }
}

/// Tasks which can be peformed by a paint worklet
pub enum PaintWorkletTask {
    DrawAPaintImage(Atom,
                    TypedSize2D<f32, CSSPixel>,
                    ScaleFactor<f32, CSSPixel, DevicePixel>,
                    Vec<(Atom, String)>,
                    Vec<String>,
                    Sender<DrawAPaintImageResult>)
}

/// A paint definition
/// https://drafts.css-houdini.org/css-paint-api/#paint-definition
/// This type is dangerous, because it contains uboxed `Heap<JSVal>` values,
/// which can't be moved.
#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
struct PaintDefinition {
    class_constructor: Heap<JSVal>,
    paint_function: Heap<JSVal>,
    constructor_valid_flag: Cell<bool>,
    context_alpha_flag: bool,
    // TODO: this should be a list of CSS syntaxes.
    input_arguments_len: usize,
    // TODO: the spec calls for fresh rendering contexts each time a paint image is drawn,
    // but to avoid having the primary worklet thread create a new renering context,
    // we recycle them.
    context: JS<PaintRenderingContext2D>,
}

impl PaintDefinition {
    fn new(class_constructor: HandleValue,
           paint_function: HandleValue,
           alpha: bool,
           input_arguments_len: usize,
           context: &PaintRenderingContext2D)
           -> Box<PaintDefinition>
    {
        let result = Box::new(PaintDefinition {
            class_constructor: Heap::default(),
            paint_function: Heap::default(),
            constructor_valid_flag: Cell::new(true),
            context_alpha_flag: alpha,
            input_arguments_len: input_arguments_len,
            context: JS::from_ref(context),
        });
        result.class_constructor.set(class_constructor.get());
        result.paint_function.set(paint_function.get());
        result
    }
}
