/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::VecDeque;
use std::ffi::CStr;
use std::rc::Rc;
use std::{mem, ptr};

use dom_struct::dom_struct;
use html5ever::{namespace_url, ns, LocalName, Namespace, Prefix};
use js::conversions::ToJSValConvertible;
use js::glue::UnwrapObjectStatic;
use js::jsapi::{HandleValueArray, Heap, IsCallable, IsConstructor, JSAutoRealm, JSObject};
use js::jsval::{BooleanValue, JSVal, NullValue, ObjectValue, UndefinedValue};
use js::rust::wrappers::{Construct1, JS_GetProperty, SameValue};
use js::rust::{HandleObject, HandleValue, MutableHandleValue};

use super::bindings::trace::HashMapTracedValues;
use crate::dom::bindings::callback::{CallbackContainer, ExceptionHandling};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CustomElementRegistryBinding::{
    CustomElementConstructor, CustomElementRegistryMethods, ElementDefinitionOptions,
};
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::conversions::{
    ConversionResult, FromJSValConvertible, StringificationBehavior,
};
use crate::dom::bindings::error::{
    report_pending_exception, throw_dom_exception, Error, ErrorResult, Fallible,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::settings_stack::is_execution_stack_empty;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::microtask::Microtask;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext};
use crate::script_thread::ScriptThread;

/// <https://dom.spec.whatwg.org/#concept-element-custom-element-state>
#[derive(Clone, Copy, Default, Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum CustomElementState {
    Undefined,
    Failed,
    #[default]
    Uncustomized,
    Precustomized,
    Custom,
}

/// <https://html.spec.whatwg.org/multipage/#customelementregistry>
#[dom_struct]
pub(crate) struct CustomElementRegistry {
    reflector_: Reflector,

    window: Dom<Window>,

    #[ignore_malloc_size_of = "Rc"]
    when_defined: DomRefCell<HashMapTracedValues<LocalName, Rc<Promise>>>,

    element_definition_is_running: Cell<bool>,

    #[ignore_malloc_size_of = "Rc"]
    definitions: DomRefCell<HashMapTracedValues<LocalName, Rc<CustomElementDefinition>>>,
}

impl CustomElementRegistry {
    fn new_inherited(window: &Window) -> CustomElementRegistry {
        CustomElementRegistry {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
            when_defined: DomRefCell::new(HashMapTracedValues::new()),
            element_definition_is_running: Cell::new(false),
            definitions: DomRefCell::new(HashMapTracedValues::new()),
        }
    }

    pub(crate) fn new(window: &Window) -> DomRoot<CustomElementRegistry> {
        reflect_dom_object(
            Box::new(CustomElementRegistry::new_inherited(window)),
            window,
            CanGc::note(),
        )
    }

    /// Cleans up any active promises
    /// <https://github.com/servo/servo/issues/15318>
    pub(crate) fn teardown(&self) {
        self.when_defined.borrow_mut().0.clear()
    }

    /// <https://html.spec.whatwg.org/multipage/#look-up-a-custom-element-definition>
    pub(crate) fn lookup_definition(
        &self,
        local_name: &LocalName,
        is: Option<&LocalName>,
    ) -> Option<Rc<CustomElementDefinition>> {
        self.definitions
            .borrow()
            .0
            .values()
            .find(|definition| {
                // Step 4-5
                definition.local_name == *local_name &&
                    (definition.name == *local_name || Some(&definition.name) == is)
            })
            .cloned()
    }

    pub(crate) fn lookup_definition_by_constructor(
        &self,
        constructor: HandleObject,
    ) -> Option<Rc<CustomElementDefinition>> {
        self.definitions
            .borrow()
            .0
            .values()
            .find(|definition| definition.constructor.callback() == constructor.get())
            .cloned()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define>
    /// Steps 10.1, 10.2
    #[allow(unsafe_code)]
    fn check_prototype(
        &self,
        constructor: HandleObject,
        prototype: MutableHandleValue,
    ) -> ErrorResult {
        unsafe {
            // Step 10.1
            if !JS_GetProperty(
                *GlobalScope::get_cx(),
                constructor,
                c"prototype".as_ptr(),
                prototype,
            ) {
                return Err(Error::JSFailed);
            }

            // Step 10.2
            if !prototype.is_object() {
                return Err(Error::Type(
                    "constructor.prototype is not an object".to_owned(),
                ));
            }
        }
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define>
    /// This function includes both steps 14.3 and 14.4 which add the callbacks to a map and
    /// process them.
    #[allow(unsafe_code)]
    unsafe fn get_callbacks(&self, prototype: HandleObject) -> Fallible<LifecycleCallbacks> {
        let cx = GlobalScope::get_cx();

        // Step 4
        Ok(LifecycleCallbacks {
            connected_callback: get_callback(cx, prototype, c"connectedCallback")?,
            disconnected_callback: get_callback(cx, prototype, c"disconnectedCallback")?,
            adopted_callback: get_callback(cx, prototype, c"adoptedCallback")?,
            attribute_changed_callback: get_callback(cx, prototype, c"attributeChangedCallback")?,

            form_associated_callback: None,
            form_disabled_callback: None,
            form_reset_callback: None,
            form_state_restore_callback: None,
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define>
    /// Step 14.13: Add form associated callbacks to LifecycleCallbacks
    #[allow(unsafe_code)]
    unsafe fn add_form_associated_callbacks(
        &self,
        prototype: HandleObject,
        callbacks: &mut LifecycleCallbacks,
    ) -> ErrorResult {
        let cx = self.window.get_cx();

        callbacks.form_associated_callback =
            get_callback(cx, prototype, c"formAssociatedCallback")?;
        callbacks.form_reset_callback = get_callback(cx, prototype, c"formResetCallback")?;
        callbacks.form_disabled_callback = get_callback(cx, prototype, c"formDisabledCallback")?;
        callbacks.form_state_restore_callback =
            get_callback(cx, prototype, c"formStateRestoreCallback")?;

        Ok(())
    }

    #[allow(unsafe_code)]
    fn get_observed_attributes(&self, constructor: HandleObject) -> Fallible<Vec<DOMString>> {
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut observed_attributes = UndefinedValue());
        if unsafe {
            !JS_GetProperty(
                *cx,
                constructor,
                c"observedAttributes".as_ptr(),
                observed_attributes.handle_mut(),
            )
        } {
            return Err(Error::JSFailed);
        }

        if observed_attributes.is_undefined() {
            return Ok(Vec::new());
        }

        let conversion = unsafe {
            FromJSValConvertible::from_jsval(
                *cx,
                observed_attributes.handle(),
                StringificationBehavior::Default,
            )
        };
        match conversion {
            Ok(ConversionResult::Success(attributes)) => Ok(attributes),
            Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.into())),
            _ => Err(Error::JSFailed),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define>
    /// Step 14.11: Get the value of `formAssociated`.
    #[allow(unsafe_code)]
    fn get_form_associated_value(&self, constructor: HandleObject) -> Fallible<bool> {
        let cx = self.window.get_cx();
        rooted!(in(*cx) let mut form_associated_value = UndefinedValue());
        if unsafe {
            !JS_GetProperty(
                *cx,
                constructor,
                c"formAssociated".as_ptr(),
                form_associated_value.handle_mut(),
            )
        } {
            return Err(Error::JSFailed);
        }

        if form_associated_value.is_undefined() {
            return Ok(false);
        }

        let conversion =
            unsafe { FromJSValConvertible::from_jsval(*cx, form_associated_value.handle(), ()) };
        match conversion {
            Ok(ConversionResult::Success(flag)) => Ok(flag),
            Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.into())),
            _ => Err(Error::JSFailed),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define>
    /// Step 14.7: Get `disabledFeatures` value
    #[allow(unsafe_code)]
    fn get_disabled_features(&self, constructor: HandleObject) -> Fallible<Vec<DOMString>> {
        let cx = self.window.get_cx();
        rooted!(in(*cx) let mut disabled_features = UndefinedValue());
        if unsafe {
            !JS_GetProperty(
                *cx,
                constructor,
                c"disabledFeatures".as_ptr(),
                disabled_features.handle_mut(),
            )
        } {
            return Err(Error::JSFailed);
        }

        if disabled_features.is_undefined() {
            return Ok(Vec::new());
        }

        let conversion = unsafe {
            FromJSValConvertible::from_jsval(
                *cx,
                disabled_features.handle(),
                StringificationBehavior::Default,
            )
        };
        match conversion {
            Ok(ConversionResult::Success(attributes)) => Ok(attributes),
            Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.into())),
            _ => Err(Error::JSFailed),
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define>
/// Step 14.4: Get `callbackValue` for all `callbackName` in `lifecycleCallbacks`.
#[allow(unsafe_code)]
fn get_callback(
    cx: JSContext,
    prototype: HandleObject,
    name: &CStr,
) -> Fallible<Option<Rc<Function>>> {
    rooted!(in(*cx) let mut callback = UndefinedValue());
    unsafe {
        // Step 10.4.1
        if !JS_GetProperty(
            *cx,
            prototype,
            name.as_ptr() as *const _,
            callback.handle_mut(),
        ) {
            return Err(Error::JSFailed);
        }

        // Step 10.4.2
        if !callback.is_undefined() {
            if !callback.is_object() || !IsCallable(callback.to_object()) {
                return Err(Error::Type("Lifecycle callback is not callable".to_owned()));
            }
            Ok(Some(Function::new(cx, callback.to_object())))
        } else {
            Ok(None)
        }
    }
}

impl CustomElementRegistryMethods<crate::DomTypeHolder> for CustomElementRegistry {
    #[allow(unsafe_code)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define>
    fn Define(
        &self,
        name: DOMString,
        constructor_: Rc<CustomElementConstructor>,
        options: &ElementDefinitionOptions,
    ) -> ErrorResult {
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let constructor = constructor_.callback());
        let name = LocalName::from(&*name);

        // Step 1
        // We must unwrap the constructor as all wrappers are constructable if they are callable.
        rooted!(in(*cx) let unwrapped_constructor = unsafe { UnwrapObjectStatic(constructor.get()) });

        if unwrapped_constructor.is_null() {
            // We do not have permission to access the unwrapped constructor.
            return Err(Error::Security);
        }

        if unsafe { !IsConstructor(unwrapped_constructor.get()) } {
            return Err(Error::Type(
                "Second argument of CustomElementRegistry.define is not a constructor".to_owned(),
            ));
        }

        // Step 2
        if !is_valid_custom_element_name(&name) {
            return Err(Error::Syntax);
        }

        // Step 3
        if self.definitions.borrow().contains_key(&name) {
            return Err(Error::NotSupported);
        }

        // Step 4
        if self
            .definitions
            .borrow()
            .iter()
            .any(|(_, def)| def.constructor == constructor_)
        {
            return Err(Error::NotSupported);
        }

        // Step 6
        let extends = &options.extends;

        // Steps 5, 7
        let local_name = if let Some(ref extended_name) = *extends {
            // Step 7.1
            if is_valid_custom_element_name(extended_name) {
                return Err(Error::NotSupported);
            }

            // Step 7.2
            if !is_extendable_element_interface(extended_name) {
                return Err(Error::NotSupported);
            }

            LocalName::from(&**extended_name)
        } else {
            // Step 7.3
            name.clone()
        };

        // Step 8
        if self.element_definition_is_running.get() {
            return Err(Error::NotSupported);
        }

        // Step 9
        self.element_definition_is_running.set(true);

        // Steps 10-13: Initialize `formAssociated`, `disableInternals`, `disableShadow`, and
        // `observedAttributes` with default values, but this is done later.

        // Steps 14.1 - 14.2: Get the value of the prototype.
        rooted!(in(*cx) let mut prototype = UndefinedValue());
        {
            let _ac = JSAutoRealm::new(*cx, constructor.get());
            if let Err(error) = self.check_prototype(constructor.handle(), prototype.handle_mut()) {
                self.element_definition_is_running.set(false);
                return Err(error);
            }
        };

        // Steps 10.3 - 10.4
        // It would be easier to get all the callbacks in one pass after
        // we know whether this definition is going to be form-associated,
        // but the order of operations is specified and it's observable
        // if one of the callback getters throws an exception.
        rooted!(in(*cx) let proto_object = prototype.to_object());
        let mut callbacks = {
            let _ac = JSAutoRealm::new(*cx, proto_object.get());
            match unsafe { self.get_callbacks(proto_object.handle()) } {
                Ok(callbacks) => callbacks,
                Err(error) => {
                    self.element_definition_is_running.set(false);
                    return Err(error);
                },
            }
        };

        // Step 14.5: Handle the case where with `attributeChangedCallback` on `lifecycleCallbacks`
        // is not null.
        let observed_attributes = if callbacks.attribute_changed_callback.is_some() {
            let _ac = JSAutoRealm::new(*cx, constructor.get());
            match self.get_observed_attributes(constructor.handle()) {
                Ok(attributes) => attributes,
                Err(error) => {
                    self.element_definition_is_running.set(false);
                    return Err(error);
                },
            }
        } else {
            Vec::new()
        };

        // Steps 14.6 - 14.10: Handle `disabledFeatures`.
        let (disable_internals, disable_shadow) = {
            let _ac = JSAutoRealm::new(*cx, constructor.get());
            match self.get_disabled_features(constructor.handle()) {
                Ok(sequence) => (
                    sequence.iter().any(|s| *s == "internals"),
                    sequence.iter().any(|s| *s == "shadow"),
                ),
                Err(error) => {
                    self.element_definition_is_running.set(false);
                    return Err(error);
                },
            }
        };

        // Step 14.11 - 14.12: Handle `formAssociated`.
        let form_associated = {
            let _ac = JSAutoRealm::new(*cx, constructor.get());
            match self.get_form_associated_value(constructor.handle()) {
                Ok(flag) => flag,
                Err(error) => {
                    self.element_definition_is_running.set(false);
                    return Err(error);
                },
            }
        };

        // Steps 14.13: Add the `formAssociated` callbacks.
        if form_associated {
            let _ac = JSAutoRealm::new(*cx, proto_object.get());
            unsafe {
                if let Err(error) =
                    self.add_form_associated_callbacks(proto_object.handle(), &mut callbacks)
                {
                    self.element_definition_is_running.set(false);
                    return Err(error);
                }
            }
        }

        self.element_definition_is_running.set(false);

        // Step 15: Set up the new custom element definition.
        let definition = Rc::new(CustomElementDefinition::new(
            name.clone(),
            local_name.clone(),
            constructor_,
            observed_attributes,
            callbacks,
            form_associated,
            disable_internals,
            disable_shadow,
        ));

        // Step 16: Add definition to this CustomElementRegistry.
        self.definitions
            .borrow_mut()
            .insert(name.clone(), definition.clone());

        // Step 17: Let document be this CustomElementRegistry's relevant global object's
        // associated Document.
        let document = self.window.Document();

        // Steps 18-19: Enqueue custom elements upgrade reaction for upgrade candidates.
        for candidate in document
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::Yes)
            .filter_map(DomRoot::downcast::<Element>)
        {
            let is = candidate.get_is();
            if *candidate.local_name() == local_name &&
                *candidate.namespace() == ns!(html) &&
                (extends.is_none() || is.as_ref() == Some(&name))
            {
                ScriptThread::enqueue_upgrade_reaction(&candidate, definition.clone());
            }
        }

        // Step 16, 16.3
        let promise = self.when_defined.borrow_mut().remove(&name);
        if let Some(promise) = promise {
            unsafe {
                rooted!(in(*cx) let mut constructor = UndefinedValue());
                definition
                    .constructor
                    .to_jsval(*cx, constructor.handle_mut());
                promise.resolve_native(&constructor.get());
            }
        }
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-get>
    #[allow(unsafe_code)]
    fn Get(&self, cx: JSContext, name: DOMString, mut retval: MutableHandleValue) {
        match self.definitions.borrow().get(&LocalName::from(&*name)) {
            Some(definition) => unsafe {
                definition.constructor.to_jsval(*cx, retval);
            },
            None => retval.set(UndefinedValue()),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-getname>
    fn GetName(&self, constructor: Rc<CustomElementConstructor>) -> Option<DOMString> {
        self.definitions
            .borrow()
            .0
            .values()
            .find(|definition| definition.constructor == constructor)
            .map(|definition| DOMString::from(definition.name.to_string()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-whendefined>
    #[allow(unsafe_code)]
    fn WhenDefined(&self, name: DOMString, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let name = LocalName::from(&*name);

        // Step 1
        if !is_valid_custom_element_name(&name) {
            let promise = Promise::new_in_current_realm(comp, can_gc);
            promise.reject_native(&DOMException::new(
                self.window.as_global_scope(),
                DOMErrorName::SyntaxError,
            ));
            return promise;
        }

        // Step 2
        if let Some(definition) = self.definitions.borrow().get(&LocalName::from(&*name)) {
            unsafe {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut constructor = UndefinedValue());
                definition
                    .constructor
                    .to_jsval(*cx, constructor.handle_mut());
                let promise = Promise::new_in_current_realm(comp, can_gc);
                promise.resolve_native(&constructor.get());
                return promise;
            }
        }

        // Steps 3, 4, 5
        let existing_promise = self.when_defined.borrow().get(&name).cloned();
        let promise = existing_promise.unwrap_or_else(|| {
            let promise = Promise::new_in_current_realm(comp, can_gc);
            self.when_defined.borrow_mut().insert(name, promise.clone());
            promise
        });

        // Step 6
        promise
    }
    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-upgrade>
    fn Upgrade(&self, node: &Node) {
        // Spec says to make a list first and then iterate the list, but
        // try-to-upgrade only queues upgrade reactions and doesn't itself
        // modify the tree, so that's not an observable distinction.
        node.traverse_preorder(ShadowIncluding::Yes).for_each(|n| {
            if let Some(element) = n.downcast::<Element>() {
                try_upgrade_element(element);
            }
        });
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) struct LifecycleCallbacks {
    #[ignore_malloc_size_of = "Rc"]
    connected_callback: Option<Rc<Function>>,

    #[ignore_malloc_size_of = "Rc"]
    disconnected_callback: Option<Rc<Function>>,

    #[ignore_malloc_size_of = "Rc"]
    adopted_callback: Option<Rc<Function>>,

    #[ignore_malloc_size_of = "Rc"]
    attribute_changed_callback: Option<Rc<Function>>,

    #[ignore_malloc_size_of = "Rc"]
    form_associated_callback: Option<Rc<Function>>,

    #[ignore_malloc_size_of = "Rc"]
    form_reset_callback: Option<Rc<Function>>,

    #[ignore_malloc_size_of = "Rc"]
    form_disabled_callback: Option<Rc<Function>>,

    #[ignore_malloc_size_of = "Rc"]
    form_state_restore_callback: Option<Rc<Function>>,
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum ConstructionStackEntry {
    Element(DomRoot<Element>),
    AlreadyConstructedMarker,
}

/// <https://html.spec.whatwg.org/multipage/#custom-element-definition>
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) struct CustomElementDefinition {
    #[no_trace]
    pub(crate) name: LocalName,

    #[no_trace]
    pub(crate) local_name: LocalName,

    #[ignore_malloc_size_of = "Rc"]
    pub(crate) constructor: Rc<CustomElementConstructor>,

    pub(crate) observed_attributes: Vec<DOMString>,

    pub(crate) callbacks: LifecycleCallbacks,

    pub(crate) construction_stack: DomRefCell<Vec<ConstructionStackEntry>>,

    pub(crate) form_associated: bool,

    pub(crate) disable_internals: bool,

    pub(crate) disable_shadow: bool,
}

impl CustomElementDefinition {
    #[allow(clippy::too_many_arguments)]
    fn new(
        name: LocalName,
        local_name: LocalName,
        constructor: Rc<CustomElementConstructor>,
        observed_attributes: Vec<DOMString>,
        callbacks: LifecycleCallbacks,
        form_associated: bool,
        disable_internals: bool,
        disable_shadow: bool,
    ) -> CustomElementDefinition {
        CustomElementDefinition {
            name,
            local_name,
            constructor,
            observed_attributes,
            callbacks,
            construction_stack: Default::default(),
            form_associated,
            disable_internals,
            disable_shadow,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#autonomous-custom-element>
    pub(crate) fn is_autonomous(&self) -> bool {
        self.name == self.local_name
    }

    /// <https://dom.spec.whatwg.org/#concept-create-element> Step 4.1
    #[allow(unsafe_code)]
    pub(crate) fn create_element(
        &self,
        document: &Document,
        prefix: Option<Prefix>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Element>> {
        let window = document.window();
        let cx = GlobalScope::get_cx();
        // Step 4.1.1. Let C be definition’s constructor.
        rooted!(in(*cx) let constructor = ObjectValue(self.constructor.callback()));
        rooted!(in(*cx) let mut element = ptr::null_mut::<JSObject>());
        {
            // Go into the constructor's realm
            let _ac = JSAutoRealm::new(*cx, self.constructor.callback());
            // Step 4.1.2. Set result to the result of constructing C, with no arguments.
            let args = HandleValueArray::empty();
            if unsafe { !Construct1(*cx, constructor.handle(), &args, element.handle_mut()) } {
                return Err(Error::JSFailed);
            }
        }

        // https://heycam.github.io/webidl/#construct-a-callback-function
        // https://html.spec.whatwg.org/multipage/#clean-up-after-running-script
        if is_execution_stack_empty() {
            window
                .as_global_scope()
                .perform_a_microtask_checkpoint(can_gc);
        }

        rooted!(in(*cx) let element_val = ObjectValue(element.get()));
        let element: DomRoot<Element> =
            match unsafe { DomRoot::from_jsval(*cx, element_val.handle(), ()) } {
                Ok(ConversionResult::Success(element)) => element,
                Ok(ConversionResult::Failure(..)) => {
                    return Err(Error::Type(
                        "Constructor did not return a DOM node".to_owned(),
                    ));
                },
                _ => return Err(Error::JSFailed),
            };

        // Step 4.1.3. Assert: result’s custom element state and custom element definition are initialized.
        // Step 4.1.4. Assert: result’s namespace is the HTML namespace.
        // Note: IDL enforces that result is an HTMLElement object, which all use the HTML namespace.
        // Note: the custom element definition is initialized by the caller if
        // this method returns a success value.
        assert!(element.is::<HTMLElement>());

        // Step 4.1.5. If result’s attribute list is not empty, then throw a "NotSupportedError" DOMException.
        // Step 4.1.6. If result has children, then throw a "NotSupportedError" DOMException.
        // Step 4.1.7. If result’s parent is not null, then throw a "NotSupportedError" DOMException.
        // Step 4.1.8. If result’s node document is not document, then throw a "NotSupportedError" DOMException.
        // Step 4.1.9. If result’s local name is not equal to localName then throw a "NotSupportedError" DOMException.
        if element.HasAttributes() ||
            element.upcast::<Node>().children_count() > 0 ||
            element.upcast::<Node>().has_parent() ||
            &*element.upcast::<Node>().owner_doc() != document ||
            *element.namespace() != ns!(html) ||
            *element.local_name() != self.local_name
        {
            return Err(Error::NotSupported);
        }

        // Step 4.1.10. Set result’s namespace prefix to prefix.
        element.set_prefix(prefix);

        // Step 4.1.11. Set result’s is value to null.
        // Element's `is` is None by default

        Ok(element)
    }
}

/// <https://html.spec.whatwg.org/multipage/#concept-upgrade-an-element>
pub(crate) fn upgrade_element(
    definition: Rc<CustomElementDefinition>,
    element: &Element,
    can_gc: CanGc,
) {
    // Step 1
    let state = element.get_custom_element_state();
    if state != CustomElementState::Undefined && state != CustomElementState::Uncustomized {
        return;
    }

    // Step 2
    element.set_custom_element_definition(Rc::clone(&definition));

    // Step 3
    element.set_custom_element_state(CustomElementState::Failed);

    // Step 4
    for attr in element.attrs().iter() {
        let local_name = attr.local_name().clone();
        let value = DOMString::from(&**attr.value());
        let namespace = attr.namespace().clone();
        ScriptThread::enqueue_callback_reaction(
            element,
            CallbackReaction::AttributeChanged(local_name, None, Some(value), namespace),
            Some(definition.clone()),
        );
    }

    // Step 5
    if element.is_connected() {
        ScriptThread::enqueue_callback_reaction(
            element,
            CallbackReaction::Connected,
            Some(definition.clone()),
        );
    }

    // Step 6
    definition
        .construction_stack
        .borrow_mut()
        .push(ConstructionStackEntry::Element(DomRoot::from_ref(element)));

    // Steps 7-8, successful case
    let result = run_upgrade_constructor(&definition, element, can_gc);

    // "regardless of whether the above steps threw an exception" step
    definition.construction_stack.borrow_mut().pop();

    // Step 8 exception handling
    if let Err(error) = result {
        // Step 8.exception.1
        element.clear_custom_element_definition();

        // Step 8.exception.2
        element.clear_reaction_queue();

        // Step 8.exception.3
        let global = GlobalScope::current().expect("No current global");
        let cx = GlobalScope::get_cx();
        let ar = enter_realm(&*global);
        throw_dom_exception(cx, &global, error);
        report_pending_exception(cx, true, InRealm::Entered(&ar), can_gc);

        return;
    }

    // Step 9: handle with form-associated custom element
    if let Some(html_element) = element.downcast::<HTMLElement>() {
        if html_element.is_form_associated_custom_element() {
            // We know this element is is form-associated, so we can use the implementation of
            // `FormControl` for HTMLElement, which makes that assumption.
            // Step 9.1: Reset the form owner of element
            html_element.reset_form_owner();
            if let Some(form) = html_element.form_owner() {
                // Even though the tree hasn't structurally mutated,
                // HTMLCollections need to be invalidated.
                form.upcast::<Node>().rev_version();
                // The spec tells us specifically to enqueue a formAssociated reaction
                // here, but it also says to do that for resetting form owner in general,
                // and we don't need two reactions.
            }

            // Either enabled_state or disabled_state needs to be set,
            // and the possibility of a disabled fieldset ancestor needs
            // to be accounted for. (In the spec, being disabled is
            // a fact that's true or false about a node at a given time,
            // not a flag that belongs to the node and is updated,
            // so it doesn't describe this check as an action.)
            element.check_disabled_attribute();
            element.check_ancestors_disabled_state_for_form_control();
            element.update_read_write_state_from_readonly_attribute();

            // Step 9.2: If element is disabled, then enqueue a custom element callback reaction
            // with element.
            if element.disabled_state() {
                ScriptThread::enqueue_callback_reaction(
                    element,
                    CallbackReaction::FormDisabled(true),
                    Some(definition.clone()),
                )
            }
        }
    }

    // Step 10
    element.set_custom_element_state(CustomElementState::Custom);
}

/// <https://html.spec.whatwg.org/multipage/#concept-upgrade-an-element>
/// Steps 8.1-8.3
#[allow(unsafe_code)]
fn run_upgrade_constructor(
    definition: &CustomElementDefinition,
    element: &Element,
    can_gc: CanGc,
) -> ErrorResult {
    let constructor = &definition.constructor;
    let window = element.owner_window();
    let cx = GlobalScope::get_cx();
    rooted!(in(*cx) let constructor_val = ObjectValue(constructor.callback()));
    rooted!(in(*cx) let mut element_val = UndefinedValue());
    unsafe {
        element.to_jsval(*cx, element_val.handle_mut());
    }
    rooted!(in(*cx) let mut construct_result = ptr::null_mut::<JSObject>());
    {
        // Step 8.1. If definition's disable shadow is true and element's shadow root is non-null,
        // then throw a "NotSupportedError" DOMException.
        if definition.disable_shadow && element.is_shadow_host() {
            return Err(Error::NotSupported);
        }

        // Go into the constructor's realm
        let _ac = JSAutoRealm::new(*cx, constructor.callback());
        let args = HandleValueArray::empty();
        // Step 8.2. Set element's custom element state to "precustomized".
        element.set_custom_element_state(CustomElementState::Precustomized);

        if unsafe {
            !Construct1(
                *cx,
                constructor_val.handle(),
                &args,
                construct_result.handle_mut(),
            )
        } {
            return Err(Error::JSFailed);
        }

        // https://heycam.github.io/webidl/#construct-a-callback-function
        // https://html.spec.whatwg.org/multipage/#clean-up-after-running-script
        if is_execution_stack_empty() {
            window
                .as_global_scope()
                .perform_a_microtask_checkpoint(can_gc);
        }

        // Step 8.3. Let constructResult be the result of constructing C, with no arguments.
        let mut same = false;
        rooted!(in(*cx) let construct_result_val = ObjectValue(construct_result.get()));
        // Step 8.4. If SameValue(constructResult, element) is false, then throw a TypeError.
        if unsafe {
            !SameValue(
                *cx,
                construct_result_val.handle(),
                element_val.handle(),
                &mut same,
            )
        } {
            return Err(Error::JSFailed);
        }
        if !same {
            return Err(Error::Type(
                "Returned element is not SameValue as the upgraded element".to_string(),
            ));
        }
    }
    Ok(())
}

/// <https://html.spec.whatwg.org/multipage/#concept-try-upgrade>
pub(crate) fn try_upgrade_element(element: &Element) {
    // Step 1
    let document = element.owner_document();
    let namespace = element.namespace();
    let local_name = element.local_name();
    let is = element.get_is();
    if let Some(definition) =
        document.lookup_custom_element_definition(namespace, local_name, is.as_ref())
    {
        // Step 2
        ScriptThread::enqueue_upgrade_reaction(element, definition);
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum CustomElementReaction {
    Upgrade(#[ignore_malloc_size_of = "Rc"] Rc<CustomElementDefinition>),
    Callback(
        #[ignore_malloc_size_of = "Rc"] Rc<Function>,
        #[ignore_malloc_size_of = "mozjs"] Box<[Heap<JSVal>]>,
    ),
}

impl CustomElementReaction {
    /// <https://html.spec.whatwg.org/multipage/#invoke-custom-element-reactions>
    #[allow(unsafe_code)]
    pub(crate) fn invoke(&self, element: &Element, can_gc: CanGc) {
        // Step 2.1
        match *self {
            CustomElementReaction::Upgrade(ref definition) => {
                upgrade_element(definition.clone(), element, can_gc)
            },
            CustomElementReaction::Callback(ref callback, ref arguments) => {
                // We're rooted, so it's safe to hand out a handle to objects in Heap
                let arguments = arguments
                    .iter()
                    .map(|arg| unsafe { HandleValue::from_raw(arg.handle()) })
                    .collect();
                rooted!(in(*GlobalScope::get_cx()) let mut value: JSVal);
                let _ = callback.Call_(
                    element,
                    arguments,
                    value.handle_mut(),
                    ExceptionHandling::Report,
                );
            },
        }
    }
}

pub(crate) enum CallbackReaction {
    Connected,
    Disconnected,
    Adopted(DomRoot<Document>, DomRoot<Document>),
    AttributeChanged(LocalName, Option<DOMString>, Option<DOMString>, Namespace),
    FormAssociated(Option<DomRoot<HTMLFormElement>>),
    FormDisabled(bool),
    FormReset,
}

/// <https://html.spec.whatwg.org/multipage/#processing-the-backup-element-queue>
#[derive(Clone, Copy, Eq, JSTraceable, MallocSizeOf, PartialEq)]
enum BackupElementQueueFlag {
    Processing,
    NotProcessing,
}

/// <https://html.spec.whatwg.org/multipage/#custom-element-reactions-stack>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct CustomElementReactionStack {
    stack: DomRefCell<Vec<ElementQueue>>,
    backup_queue: ElementQueue,
    processing_backup_element_queue: Cell<BackupElementQueueFlag>,
}

impl CustomElementReactionStack {
    pub(crate) fn new() -> CustomElementReactionStack {
        CustomElementReactionStack {
            stack: DomRefCell::new(Vec::new()),
            backup_queue: ElementQueue::new(),
            processing_backup_element_queue: Cell::new(BackupElementQueueFlag::NotProcessing),
        }
    }

    pub(crate) fn push_new_element_queue(&self) {
        self.stack.borrow_mut().push(ElementQueue::new());
    }

    pub(crate) fn pop_current_element_queue(&self, can_gc: CanGc) {
        rooted_vec!(let mut stack);
        mem::swap(&mut *stack, &mut *self.stack.borrow_mut());

        if let Some(current_queue) = stack.last() {
            current_queue.invoke_reactions(can_gc);
        }
        stack.pop();

        mem::swap(&mut *self.stack.borrow_mut(), &mut *stack);
        self.stack.borrow_mut().append(&mut *stack);
    }

    /// <https://html.spec.whatwg.org/multipage/#enqueue-an-element-on-the-appropriate-element-queue>
    /// Step 4
    pub(crate) fn invoke_backup_element_queue(&self, can_gc: CanGc) {
        // Step 4.1
        self.backup_queue.invoke_reactions(can_gc);

        // Step 4.2
        self.processing_backup_element_queue
            .set(BackupElementQueueFlag::NotProcessing);
    }

    /// <https://html.spec.whatwg.org/multipage/#enqueue-an-element-on-the-appropriate-element-queue>
    pub(crate) fn enqueue_element(&self, element: &Element) {
        if let Some(current_queue) = self.stack.borrow().last() {
            // Step 2
            current_queue.append_element(element);
        } else {
            // Step 1.1
            self.backup_queue.append_element(element);

            // Step 1.2
            if self.processing_backup_element_queue.get() == BackupElementQueueFlag::Processing {
                return;
            }

            // Step 1.3
            self.processing_backup_element_queue
                .set(BackupElementQueueFlag::Processing);

            // Step 4
            ScriptThread::enqueue_microtask(Microtask::CustomElementReaction);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#enqueue-a-custom-element-callback-reaction>
    #[allow(unsafe_code)]
    pub(crate) fn enqueue_callback_reaction(
        &self,
        element: &Element,
        reaction: CallbackReaction,
        definition: Option<Rc<CustomElementDefinition>>,
    ) {
        // Step 1
        let definition = match definition.or_else(|| element.get_custom_element_definition()) {
            Some(definition) => definition,
            None => return,
        };

        // Step 2
        let (callback, args) = match reaction {
            CallbackReaction::Connected => {
                (definition.callbacks.connected_callback.clone(), Vec::new())
            },
            CallbackReaction::Disconnected => (
                definition.callbacks.disconnected_callback.clone(),
                Vec::new(),
            ),
            CallbackReaction::Adopted(ref old_doc, ref new_doc) => {
                let args = vec![Heap::default(), Heap::default()];
                args[0].set(ObjectValue(old_doc.reflector().get_jsobject().get()));
                args[1].set(ObjectValue(new_doc.reflector().get_jsobject().get()));
                (definition.callbacks.adopted_callback.clone(), args)
            },
            CallbackReaction::AttributeChanged(local_name, old_val, val, namespace) => {
                // Step 4
                if !definition
                    .observed_attributes
                    .iter()
                    .any(|attr| *attr == *local_name)
                {
                    return;
                }

                let cx = GlobalScope::get_cx();
                // We might be here during HTML parsing, rather than
                // during Javscript execution, and so we typically aren't
                // already in a realm here.
                let _ac = JSAutoRealm::new(*cx, element.global().reflector().get_jsobject().get());

                let local_name = DOMString::from(&*local_name);
                rooted!(in(*cx) let mut name_value = UndefinedValue());
                unsafe {
                    local_name.to_jsval(*cx, name_value.handle_mut());
                }

                rooted!(in(*cx) let mut old_value = NullValue());
                if let Some(old_val) = old_val {
                    unsafe {
                        old_val.to_jsval(*cx, old_value.handle_mut());
                    }
                }

                rooted!(in(*cx) let mut value = NullValue());
                if let Some(val) = val {
                    unsafe {
                        val.to_jsval(*cx, value.handle_mut());
                    }
                }

                rooted!(in(*cx) let mut namespace_value = NullValue());
                if namespace != ns!() {
                    let namespace = DOMString::from(&*namespace);
                    unsafe {
                        namespace.to_jsval(*cx, namespace_value.handle_mut());
                    }
                }

                let args = vec![
                    Heap::default(),
                    Heap::default(),
                    Heap::default(),
                    Heap::default(),
                ];
                args[0].set(name_value.get());
                args[1].set(old_value.get());
                args[2].set(value.get());
                args[3].set(namespace_value.get());

                (
                    definition.callbacks.attribute_changed_callback.clone(),
                    args,
                )
            },
            CallbackReaction::FormAssociated(form) => {
                let args = vec![Heap::default()];
                if let Some(form) = form {
                    args[0].set(ObjectValue(form.reflector().get_jsobject().get()));
                } else {
                    args[0].set(NullValue());
                }
                (definition.callbacks.form_associated_callback.clone(), args)
            },
            CallbackReaction::FormDisabled(disabled) => {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut disabled_value = BooleanValue(disabled));
                let args = vec![Heap::default()];
                args[0].set(disabled_value.get());
                (definition.callbacks.form_disabled_callback.clone(), args)
            },
            CallbackReaction::FormReset => {
                (definition.callbacks.form_reset_callback.clone(), Vec::new())
            },
        };

        // Step 3
        let callback = match callback {
            Some(callback) => callback,
            None => return,
        };

        // Step 5
        element.push_callback_reaction(callback, args.into_boxed_slice());

        // Step 6
        self.enqueue_element(element);
    }

    /// <https://html.spec.whatwg.org/multipage/#enqueue-a-custom-element-upgrade-reaction>
    pub(crate) fn enqueue_upgrade_reaction(
        &self,
        element: &Element,
        definition: Rc<CustomElementDefinition>,
    ) {
        // Step 1
        element.push_upgrade_reaction(definition);
        // Step 2
        self.enqueue_element(element);
    }
}

/// <https://html.spec.whatwg.org/multipage/#element-queue>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct ElementQueue {
    queue: DomRefCell<VecDeque<Dom<Element>>>,
}

impl ElementQueue {
    fn new() -> ElementQueue {
        ElementQueue {
            queue: Default::default(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#invoke-custom-element-reactions>
    fn invoke_reactions(&self, can_gc: CanGc) {
        // Steps 1-2
        while let Some(element) = self.next_element() {
            element.invoke_reactions(can_gc)
        }
        self.queue.borrow_mut().clear();
    }

    fn next_element(&self) -> Option<DomRoot<Element>> {
        self.queue
            .borrow_mut()
            .pop_front()
            .as_deref()
            .map(DomRoot::from_ref)
    }

    fn append_element(&self, element: &Element) {
        self.queue.borrow_mut().push_back(Dom::from_ref(element));
    }
}

/// <https://html.spec.whatwg.org/multipage/#valid-custom-element-name>
pub(crate) fn is_valid_custom_element_name(name: &str) -> bool {
    // Custom elment names must match:
    // PotentialCustomElementName ::= [a-z] (PCENChar)* '-' (PCENChar)*

    let mut chars = name.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_lowercase()) {
        return false;
    }

    let mut has_dash = false;

    for c in chars {
        if c == '-' {
            has_dash = true;
            continue;
        }

        if !is_potential_custom_element_char(c) {
            return false;
        }
    }

    if !has_dash {
        return false;
    }

    if name == "annotation-xml" ||
        name == "color-profile" ||
        name == "font-face" ||
        name == "font-face-src" ||
        name == "font-face-uri" ||
        name == "font-face-format" ||
        name == "font-face-name" ||
        name == "missing-glyph"
    {
        return false;
    }

    true
}

/// Check if this character is a PCENChar
/// <https://html.spec.whatwg.org/multipage/#prod-pcenchar>
fn is_potential_custom_element_char(c: char) -> bool {
    c == '-' ||
        c == '.' ||
        c == '_' ||
        c == '\u{B7}' ||
        c.is_ascii_digit() ||
        c.is_ascii_lowercase() ||
        ('\u{C0}'..='\u{D6}').contains(&c) ||
        ('\u{D8}'..='\u{F6}').contains(&c) ||
        ('\u{F8}'..='\u{37D}').contains(&c) ||
        ('\u{37F}'..='\u{1FFF}').contains(&c) ||
        ('\u{200C}'..='\u{200D}').contains(&c) ||
        ('\u{203F}'..='\u{2040}').contains(&c) ||
        ('\u{2070}'..='\u{218F}').contains(&c) ||
        ('\u{2C00}'..='\u{2FEF}').contains(&c) ||
        ('\u{3001}'..='\u{D7FF}').contains(&c) ||
        ('\u{F900}'..='\u{FDCF}').contains(&c) ||
        ('\u{FDF0}'..='\u{FFFD}').contains(&c) ||
        ('\u{10000}'..='\u{EFFFF}').contains(&c)
}

fn is_extendable_element_interface(element: &str) -> bool {
    element == "a" ||
        element == "abbr" ||
        element == "acronym" ||
        element == "address" ||
        element == "area" ||
        element == "article" ||
        element == "aside" ||
        element == "audio" ||
        element == "b" ||
        element == "base" ||
        element == "bdi" ||
        element == "bdo" ||
        element == "big" ||
        element == "blockquote" ||
        element == "body" ||
        element == "br" ||
        element == "button" ||
        element == "canvas" ||
        element == "caption" ||
        element == "center" ||
        element == "cite" ||
        element == "code" ||
        element == "col" ||
        element == "colgroup" ||
        element == "data" ||
        element == "datalist" ||
        element == "dd" ||
        element == "del" ||
        element == "details" ||
        element == "dfn" ||
        element == "dialog" ||
        element == "dir" ||
        element == "div" ||
        element == "dl" ||
        element == "dt" ||
        element == "em" ||
        element == "embed" ||
        element == "fieldset" ||
        element == "figcaption" ||
        element == "figure" ||
        element == "font" ||
        element == "footer" ||
        element == "form" ||
        element == "frame" ||
        element == "frameset" ||
        element == "h1" ||
        element == "h2" ||
        element == "h3" ||
        element == "h4" ||
        element == "h5" ||
        element == "h6" ||
        element == "head" ||
        element == "header" ||
        element == "hgroup" ||
        element == "hr" ||
        element == "html" ||
        element == "i" ||
        element == "iframe" ||
        element == "img" ||
        element == "input" ||
        element == "ins" ||
        element == "kbd" ||
        element == "label" ||
        element == "legend" ||
        element == "li" ||
        element == "link" ||
        element == "listing" ||
        element == "main" ||
        element == "map" ||
        element == "mark" ||
        element == "marquee" ||
        element == "menu" ||
        element == "meta" ||
        element == "meter" ||
        element == "nav" ||
        element == "nobr" ||
        element == "noframes" ||
        element == "noscript" ||
        element == "object" ||
        element == "ol" ||
        element == "optgroup" ||
        element == "option" ||
        element == "output" ||
        element == "p" ||
        element == "param" ||
        element == "picture" ||
        element == "plaintext" ||
        element == "pre" ||
        element == "progress" ||
        element == "q" ||
        element == "rp" ||
        element == "rt" ||
        element == "ruby" ||
        element == "s" ||
        element == "samp" ||
        element == "script" ||
        element == "section" ||
        element == "select" ||
        element == "small" ||
        element == "source" ||
        element == "span" ||
        element == "strike" ||
        element == "strong" ||
        element == "style" ||
        element == "sub" ||
        element == "summary" ||
        element == "sup" ||
        element == "table" ||
        element == "tbody" ||
        element == "td" ||
        element == "template" ||
        element == "textarea" ||
        element == "tfoot" ||
        element == "th" ||
        element == "thead" ||
        element == "time" ||
        element == "title" ||
        element == "tr" ||
        element == "tt" ||
        element == "track" ||
        element == "u" ||
        element == "ul" ||
        element == "var" ||
        element == "video" ||
        element == "wbr" ||
        element == "xmp"
}
