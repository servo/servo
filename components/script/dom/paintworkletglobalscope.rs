/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::hash_map::Entry;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use base::id::PipelineId;
use crossbeam_channel::{unbounded, Sender};
use dom_struct::dom_struct;
use euclid::{Scale, Size2D};
use js::jsapi::{
    HandleValueArray, Heap, IsCallable, IsConstructor, JSAutoRealm, JSObject,
    JS_ClearPendingException, JS_IsExceptionPending, NewArrayObject, Value,
};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::rust::wrappers::{Call, Construct1};
use js::rust::{HandleValue, Runtime};
use net_traits::image_cache::ImageCache;
use pixels::PixelFormat;
use profile_traits::ipc;
use script_traits::{DrawAPaintImageResult, PaintWorkletError, Painter};
use servo_atoms::Atom;
use servo_config::pref;
use servo_url::ServoUrl;
use style_traits::{CSSPixel, SpeculativePainter};
use webrender_api::units::DevicePixel;

use super::bindings::trace::HashMapTracedValues;
use crate::dom::bindings::callback::CallbackContainer;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::PaintWorkletGlobalScopeBinding;
use crate::dom::bindings::codegen::Bindings::PaintWorkletGlobalScopeBinding::PaintWorkletGlobalScopeMethods;
use crate::dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use crate::dom::bindings::conversions::{get_property, get_property_jsval};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssstylevalue::CSSStyleValue;
use crate::dom::paintrenderingcontext2d::PaintRenderingContext2D;
use crate::dom::paintsize::PaintSize;
use crate::dom::stylepropertymapreadonly::StylePropertyMapReadOnly;
use crate::dom::worklet::WorkletExecutor;
use crate::dom::workletglobalscope::{WorkletGlobalScope, WorkletGlobalScopeInit, WorkletTask};
use crate::script_runtime::{CanGc, JSContext};

/// <https://drafts.css-houdini.org/css-paint-api/#paintworkletglobalscope>
#[dom_struct]
pub(crate) struct PaintWorkletGlobalScope {
    /// The worklet global for this object
    worklet_global: WorkletGlobalScope,
    /// The image cache
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    image_cache: Arc<dyn ImageCache>,
    /// <https://drafts.css-houdini.org/css-paint-api/#paint-definitions>
    paint_definitions: DomRefCell<HashMapTracedValues<Atom, Box<PaintDefinition>>>,
    /// <https://drafts.css-houdini.org/css-paint-api/#paint-class-instances>
    #[ignore_malloc_size_of = "mozjs"]
    paint_class_instances: DomRefCell<HashMapTracedValues<Atom, Box<Heap<JSVal>>>>,
    /// The most recent name the worklet was called with
    #[no_trace]
    cached_name: DomRefCell<Atom>,
    /// The most recent size the worklet was drawn at
    #[no_trace]
    cached_size: Cell<Size2D<f32, CSSPixel>>,
    /// The most recent device pixel ratio the worklet was drawn at
    #[no_trace]
    cached_device_pixel_ratio: Cell<Scale<f32, CSSPixel, DevicePixel>>,
    /// The most recent properties the worklet was drawn at
    #[no_trace]
    cached_properties: DomRefCell<Vec<(Atom, String)>>,
    /// The most recent arguments the worklet was drawn at
    cached_arguments: DomRefCell<Vec<String>>,
    /// The most recent result
    #[no_trace]
    cached_result: DomRefCell<DrawAPaintImageResult>,
}

impl PaintWorkletGlobalScope {
    #[allow(unsafe_code)]
    pub(crate) fn new(
        runtime: &Runtime,
        pipeline_id: PipelineId,
        base_url: ServoUrl,
        executor: WorkletExecutor,
        init: &WorkletGlobalScopeInit,
    ) -> DomRoot<PaintWorkletGlobalScope> {
        debug!(
            "Creating paint worklet global scope for pipeline {}.",
            pipeline_id
        );
        let global = Box::new(PaintWorkletGlobalScope {
            worklet_global: WorkletGlobalScope::new_inherited(
                pipeline_id,
                base_url,
                executor,
                init,
            ),
            image_cache: init.image_cache.clone(),
            paint_definitions: Default::default(),
            paint_class_instances: Default::default(),
            cached_name: DomRefCell::new(Atom::from("")),
            cached_size: Cell::new(Size2D::zero()),
            cached_device_pixel_ratio: Cell::new(Scale::new(1.0)),
            cached_properties: Default::default(),
            cached_arguments: Default::default(),
            cached_result: DomRefCell::new(DrawAPaintImageResult {
                width: 0,
                height: 0,
                format: PixelFormat::BGRA8,
                image_key: None,
                missing_image_urls: Vec::new(),
            }),
        });
        unsafe {
            PaintWorkletGlobalScopeBinding::Wrap::<crate::DomTypeHolder>(
                JSContext::from_ptr(runtime.cx()),
                global,
            )
        }
    }

    pub(crate) fn image_cache(&self) -> Arc<dyn ImageCache> {
        self.image_cache.clone()
    }

    pub(crate) fn perform_a_worklet_task(&self, task: PaintWorkletTask) {
        match task {
            PaintWorkletTask::DrawAPaintImage(
                name,
                size,
                device_pixel_ratio,
                properties,
                arguments,
                sender,
            ) => {
                let cache_hit = (*self.cached_name.borrow() == name) &&
                    (self.cached_size.get() == size) &&
                    (self.cached_device_pixel_ratio.get() == device_pixel_ratio) &&
                    (*self.cached_properties.borrow() == properties) &&
                    (*self.cached_arguments.borrow() == arguments);
                let result = if cache_hit {
                    debug!("Cache hit on paint worklet {}!", name);
                    self.cached_result.borrow().clone()
                } else {
                    debug!("Cache miss on paint worklet {}!", name);
                    let map = StylePropertyMapReadOnly::from_iter(
                        self.upcast(),
                        properties.iter().cloned(),
                        CanGc::note(),
                    );
                    let result =
                        self.draw_a_paint_image(&name, size, device_pixel_ratio, &map, &arguments);
                    if (result.image_key.is_some()) && (result.missing_image_urls.is_empty()) {
                        *self.cached_name.borrow_mut() = name;
                        self.cached_size.set(size);
                        self.cached_device_pixel_ratio.set(device_pixel_ratio);
                        *self.cached_properties.borrow_mut() = properties;
                        *self.cached_arguments.borrow_mut() = arguments;
                        *self.cached_result.borrow_mut() = result.clone();
                    }
                    result
                };
                let _ = sender.send(result);
            },
            PaintWorkletTask::SpeculativelyDrawAPaintImage(name, properties, arguments) => {
                let should_speculate = (*self.cached_name.borrow() != name) ||
                    (*self.cached_properties.borrow() != properties) ||
                    (*self.cached_arguments.borrow() != arguments);
                if should_speculate {
                    let size = self.cached_size.get();
                    let device_pixel_ratio = self.cached_device_pixel_ratio.get();
                    let map = StylePropertyMapReadOnly::from_iter(
                        self.upcast(),
                        properties.iter().cloned(),
                        CanGc::note(),
                    );
                    let result =
                        self.draw_a_paint_image(&name, size, device_pixel_ratio, &map, &arguments);
                    if (result.image_key.is_some()) && (result.missing_image_urls.is_empty()) {
                        *self.cached_name.borrow_mut() = name;
                        *self.cached_properties.borrow_mut() = properties;
                        *self.cached_arguments.borrow_mut() = arguments;
                        *self.cached_result.borrow_mut() = result;
                    }
                }
            },
        }
    }

    /// <https://drafts.css-houdini.org/css-paint-api/#draw-a-paint-image>
    fn draw_a_paint_image(
        &self,
        name: &Atom,
        size_in_px: Size2D<f32, CSSPixel>,
        device_pixel_ratio: Scale<f32, CSSPixel, DevicePixel>,
        properties: &StylePropertyMapReadOnly,
        arguments: &[String],
    ) -> DrawAPaintImageResult {
        let size_in_dpx = size_in_px * device_pixel_ratio;
        let size_in_dpx = Size2D::new(
            size_in_dpx.width.abs() as u32,
            size_in_dpx.height.abs() as u32,
        );

        // TODO: Steps 1-5.

        // TODO: document paint definitions.
        self.invoke_a_paint_callback(
            name,
            size_in_px,
            size_in_dpx,
            device_pixel_ratio,
            properties,
            arguments,
        )
    }

    /// <https://drafts.css-houdini.org/css-paint-api/#invoke-a-paint-callback>
    #[allow(unsafe_code)]
    fn invoke_a_paint_callback(
        &self,
        name: &Atom,
        size_in_px: Size2D<f32, CSSPixel>,
        size_in_dpx: Size2D<u32, DevicePixel>,
        device_pixel_ratio: Scale<f32, CSSPixel, DevicePixel>,
        properties: &StylePropertyMapReadOnly,
        arguments: &[String],
    ) -> DrawAPaintImageResult {
        debug!(
            "Invoking a paint callback {}({},{}) at {:?}.",
            name, size_in_px.width, size_in_px.height, device_pixel_ratio
        );

        let cx = WorkletGlobalScope::get_cx();
        let _ac = JSAutoRealm::new(*cx, self.worklet_global.reflector().get_jsobject().get());

        // TODO: Steps 1-2.1.
        // Step 2.2-5.1.
        rooted!(in(*cx) let mut class_constructor = UndefinedValue());
        rooted!(in(*cx) let mut paint_function = UndefinedValue());
        let rendering_context = match self.paint_definitions.borrow().get(name) {
            None => {
                // Step 2.2.
                warn!("Drawing un-registered paint definition {}.", name);
                return self.invalid_image(size_in_dpx, vec![]);
            },
            Some(definition) => {
                // Step 5.1
                if !definition.constructor_valid_flag.get() {
                    debug!("Drawing invalid paint definition {}.", name);
                    return self.invalid_image(size_in_dpx, vec![]);
                }
                class_constructor.set(definition.class_constructor.get());
                paint_function.set(definition.paint_function.get());
                DomRoot::from_ref(&*definition.context)
            },
        };

        // Steps 5.2-5.4
        // TODO: the spec requires calling the constructor now, but we might want to
        // prepopulate the paint instance in `RegisterPaint`, to avoid calling it in
        // the primary worklet thread.
        // https://github.com/servo/servo/issues/17377
        rooted!(in(*cx) let mut paint_instance = UndefinedValue());
        match self.paint_class_instances.borrow_mut().entry(name.clone()) {
            Entry::Occupied(entry) => paint_instance.set(entry.get().get()),
            Entry::Vacant(entry) => {
                // Step 5.2-5.3
                let args = HandleValueArray::empty();
                rooted!(in(*cx) let mut result = null_mut::<JSObject>());
                unsafe {
                    Construct1(*cx, class_constructor.handle(), &args, result.handle_mut());
                }
                paint_instance.set(ObjectValue(result.get()));
                if unsafe { JS_IsExceptionPending(*cx) } {
                    debug!("Paint constructor threw an exception {}.", name);
                    unsafe {
                        JS_ClearPendingException(*cx);
                    }
                    self.paint_definitions
                        .borrow_mut()
                        .get_mut(name)
                        .expect("Vanishing paint definition.")
                        .constructor_valid_flag
                        .set(false);
                    return self.invalid_image(size_in_dpx, vec![]);
                }
                // Step 5.4
                entry
                    .insert(Box::<Heap<Value>>::default())
                    .set(paint_instance.get());
            },
        };

        // TODO: Steps 6-7
        // Step 8
        // TODO: the spec requires creating a new paint rendering context each time,
        // this code recycles the same one.
        rendering_context.set_bitmap_dimensions(size_in_px, device_pixel_ratio);

        // Step 9
        let paint_size = PaintSize::new(self, size_in_px, CanGc::note());

        // TODO: Step 10
        // Steps 11-12
        debug!("Invoking paint function {}.", name);
        rooted_vec!(let mut arguments_values);
        for argument in arguments {
            let style_value = CSSStyleValue::new(self.upcast(), argument.clone(), CanGc::note());
            arguments_values.push(ObjectValue(style_value.reflector().get_jsobject().get()));
        }
        let arguments_value_array = HandleValueArray::from(&arguments_values);
        rooted!(in(*cx) let argument_object = unsafe { NewArrayObject(*cx, &arguments_value_array) });

        rooted_vec!(let mut callback_args);
        callback_args.push(ObjectValue(
            rendering_context.reflector().get_jsobject().get(),
        ));
        callback_args.push(ObjectValue(paint_size.reflector().get_jsobject().get()));
        callback_args.push(ObjectValue(properties.reflector().get_jsobject().get()));
        callback_args.push(ObjectValue(argument_object.get()));
        let args = HandleValueArray::from(&callback_args);

        rooted!(in(*cx) let mut result = UndefinedValue());
        unsafe {
            Call(
                *cx,
                paint_instance.handle(),
                paint_function.handle(),
                &args,
                result.handle_mut(),
            );
        }
        let missing_image_urls = rendering_context.take_missing_image_urls();

        // Step 13.
        if unsafe { JS_IsExceptionPending(*cx) } {
            debug!("Paint function threw an exception {}.", name);
            unsafe {
                JS_ClearPendingException(*cx);
            }
            return self.invalid_image(size_in_dpx, missing_image_urls);
        }

        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone())
            .expect("IPC channel creation.");
        rendering_context.send_data(sender);
        let image_key = match receiver.recv() {
            Ok(data) => Some(data.image_key),
            _ => None,
        };

        DrawAPaintImageResult {
            width: size_in_dpx.width,
            height: size_in_dpx.height,
            format: PixelFormat::BGRA8,
            image_key,
            missing_image_urls,
        }
    }

    // https://drafts.csswg.org/css-images-4/#invalid-image
    fn invalid_image(
        &self,
        size: Size2D<u32, DevicePixel>,
        missing_image_urls: Vec<ServoUrl>,
    ) -> DrawAPaintImageResult {
        debug!("Returning an invalid image.");
        DrawAPaintImageResult {
            width: size.width,
            height: size.height,
            format: PixelFormat::BGRA8,
            image_key: None,
            missing_image_urls,
        }
    }

    fn painter(&self, name: Atom) -> Box<dyn Painter> {
        // Rather annoyingly we have to use a mutex here to make the painter Sync.
        struct WorkletPainter {
            name: Atom,
            executor: Mutex<WorkletExecutor>,
        }
        impl SpeculativePainter for WorkletPainter {
            fn speculatively_draw_a_paint_image(
                &self,
                properties: Vec<(Atom, String)>,
                arguments: Vec<String>,
            ) {
                let name = self.name.clone();
                let task =
                    PaintWorkletTask::SpeculativelyDrawAPaintImage(name, properties, arguments);
                self.executor
                    .lock()
                    .expect("Locking a painter.")
                    .schedule_a_worklet_task(WorkletTask::Paint(task));
            }
        }
        impl Painter for WorkletPainter {
            fn draw_a_paint_image(
                &self,
                size: Size2D<f32, CSSPixel>,
                device_pixel_ratio: Scale<f32, CSSPixel, DevicePixel>,
                properties: Vec<(Atom, String)>,
                arguments: Vec<String>,
            ) -> Result<DrawAPaintImageResult, PaintWorkletError> {
                let name = self.name.clone();
                let (sender, receiver) = unbounded();
                let task = PaintWorkletTask::DrawAPaintImage(
                    name,
                    size,
                    device_pixel_ratio,
                    properties,
                    arguments,
                    sender,
                );
                self.executor
                    .lock()
                    .expect("Locking a painter.")
                    .schedule_a_worklet_task(WorkletTask::Paint(task));

                let timeout = pref!(dom_worklet_timeout_ms) as u64;

                receiver
                    .recv_timeout(Duration::from_millis(timeout))
                    .map_err(PaintWorkletError::from)
            }
        }
        Box::new(WorkletPainter {
            name,
            executor: Mutex::new(self.worklet_global.executor()),
        })
    }
}

/// Tasks which can be peformed by a paint worklet
pub(crate) enum PaintWorkletTask {
    DrawAPaintImage(
        Atom,
        Size2D<f32, CSSPixel>,
        Scale<f32, CSSPixel, DevicePixel>,
        Vec<(Atom, String)>,
        Vec<String>,
        Sender<DrawAPaintImageResult>,
    ),
    SpeculativelyDrawAPaintImage(Atom, Vec<(Atom, String)>, Vec<String>),
}

/// A paint definition
/// <https://drafts.css-houdini.org/css-paint-api/#paint-definition>
/// This type is dangerous, because it contains uboxed `Heap<JSVal>` values,
/// which can't be moved.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct PaintDefinition {
    #[ignore_malloc_size_of = "mozjs"]
    class_constructor: Heap<JSVal>,
    #[ignore_malloc_size_of = "mozjs"]
    paint_function: Heap<JSVal>,
    constructor_valid_flag: Cell<bool>,
    context_alpha_flag: bool,
    // TODO: this should be a list of CSS syntaxes.
    input_arguments_len: usize,
    // TODO: the spec calls for fresh rendering contexts each time a paint image is drawn,
    // but to avoid having the primary worklet thread create a new renering context,
    // we recycle them.
    context: Dom<PaintRenderingContext2D>,
}

impl PaintDefinition {
    fn new(
        class_constructor: HandleValue,
        paint_function: HandleValue,
        alpha: bool,
        input_arguments_len: usize,
        context: &PaintRenderingContext2D,
    ) -> Box<PaintDefinition> {
        let result = Box::new(PaintDefinition {
            class_constructor: Heap::default(),
            paint_function: Heap::default(),
            constructor_valid_flag: Cell::new(true),
            context_alpha_flag: alpha,
            input_arguments_len,
            context: Dom::from_ref(context),
        });
        result.class_constructor.set(class_constructor.get());
        result.paint_function.set(paint_function.get());
        result
    }
}

impl PaintWorkletGlobalScopeMethods<crate::DomTypeHolder> for PaintWorkletGlobalScope {
    #[allow(unsafe_code)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    /// <https://drafts.css-houdini.org/css-paint-api/#dom-paintworkletglobalscope-registerpaint>
    fn RegisterPaint(&self, name: DOMString, paint_ctor: Rc<VoidFunction>) -> Fallible<()> {
        let name = Atom::from(name);
        let cx = WorkletGlobalScope::get_cx();
        rooted!(in(*cx) let paint_obj = paint_ctor.callback_holder().get());
        rooted!(in(*cx) let paint_val = ObjectValue(paint_obj.get()));

        debug!("Registering paint image name {}.", name);

        // Step 1.
        if name.is_empty() {
            return Err(Error::Type(String::from("Empty paint name.")));
        }

        // Step 2-3.
        if self.paint_definitions.borrow().contains_key(&name) {
            return Err(Error::InvalidModification);
        }

        // Step 4-6.
        let mut property_names: Vec<String> =
            unsafe { get_property(*cx, paint_obj.handle(), "inputProperties", ()) }?
                .unwrap_or_default();
        let properties = property_names.drain(..).map(Atom::from).collect();

        // Step 7-9.
        let input_arguments: Vec<String> =
            unsafe { get_property(*cx, paint_obj.handle(), "inputArguments", ()) }?
                .unwrap_or_default();

        // TODO: Steps 10-11.

        // Steps 12-13.
        let alpha: bool =
            unsafe { get_property(*cx, paint_obj.handle(), "alpha", ()) }?.unwrap_or(true);

        // Step 14
        if unsafe { !IsConstructor(paint_obj.get()) } {
            return Err(Error::Type(String::from("Not a constructor.")));
        }

        // Steps 15-16
        rooted!(in(*cx) let mut prototype = UndefinedValue());
        unsafe {
            get_property_jsval(*cx, paint_obj.handle(), "prototype", prototype.handle_mut())?;
        }
        if !prototype.is_object() {
            return Err(Error::Type(String::from("Prototype is not an object.")));
        }
        rooted!(in(*cx) let prototype = prototype.to_object());

        // Steps 17-18
        rooted!(in(*cx) let mut paint_function = UndefinedValue());
        unsafe {
            get_property_jsval(
                *cx,
                prototype.handle(),
                "paint",
                paint_function.handle_mut(),
            )?;
        }
        if !paint_function.is_object() || unsafe { !IsCallable(paint_function.to_object()) } {
            return Err(Error::Type(String::from("Paint function is not callable.")));
        }

        // Step 19.
        let context = PaintRenderingContext2D::new(self, CanGc::note());
        let definition = PaintDefinition::new(
            paint_val.handle(),
            paint_function.handle(),
            alpha,
            input_arguments.len(),
            &context,
        );

        // Step 20.
        debug!("Registering definition {}.", name);
        self.paint_definitions
            .borrow_mut()
            .insert(name.clone(), definition);

        // TODO: Step 21.

        // Inform layout that there is a registered paint worklet.
        // TODO: layout will end up getting this message multiple times.
        let painter = self.painter(name.clone());
        self.worklet_global
            .register_paint_worklet(name, properties, painter);

        Ok(())
    }

    /// This is a blocking sleep function available in the paint worklet
    /// global scope behind the dom.worklet.enabled +
    /// dom.worklet.blockingsleep.enabled prefs. It is to be used only for
    /// testing, e.g., timeouts, where otherwise one would need busy waiting
    /// to make sure a certain timeout is triggered.
    /// check-tidy: no specs after this line
    fn Sleep(&self, ms: u64) {
        thread::sleep(Duration::from_millis(ms));
    }
}
