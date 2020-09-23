/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::callback::{CallbackContainer, ExceptionHandling};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CustomElementRegistryBinding::CustomElementConstructor;
use crate::dom::bindings::codegen::Bindings::CustomElementRegistryBinding::CustomElementRegistryMethods;
use crate::dom::bindings::codegen::Bindings::CustomElementRegistryBinding::ElementDefinitionOptions;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use crate::dom::bindings::conversions::{
    ConversionResult, FromJSValConvertible, StringificationBehavior,
};
use crate::dom::bindings::error::{
    report_pending_exception, throw_dom_exception, Error, ErrorResult, Fallible,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::settings_stack::is_execution_stack_empty;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{document_from_node, window_from_node, Node, ShadowIncluding};
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::microtask::Microtask;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::JSContext;
use crate::script_thread::ScriptThread;
use dom_struct::dom_struct;
use html5ever::{LocalName, Namespace, Prefix};
use js::conversions::ToJSValConvertible;
use js::glue::UnwrapObjectStatic;
use js::jsapi::{HandleValueArray, Heap, IsCallable, IsConstructor};
use js::jsapi::{JSAutoRealm, JSObject};
use js::jsval::{JSVal, NullValue, ObjectValue, UndefinedValue};
use js::rust::wrappers::{Construct1, JS_GetProperty, SameValue};
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::mem;
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

/// <https://dom.spec.whatwg.org/#concept-element-custom-element-state>
#[derive(Clone, Copy, Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub enum CustomElementState {
    Undefined,
    Failed,
    Uncustomized,
    Custom,
}

impl Default for CustomElementState {
    fn default() -> CustomElementState {
        CustomElementState::Uncustomized
    }
}

/// <https://html.spec.whatwg.org/multipage/#customelementregistry>
#[dom_struct]
pub struct CustomElementRegistry {
    reflector_: Reflector,

    window: Dom<Window>,

    #[ignore_malloc_size_of = "Rc"]
    when_defined: DomRefCell<HashMap<LocalName, Rc<Promise>>>,

    element_definition_is_running: Cell<bool>,

    #[ignore_malloc_size_of = "Rc"]
    definitions: DomRefCell<HashMap<LocalName, Rc<CustomElementDefinition>>>,
}

impl CustomElementRegistry {
    fn new_inherited(window: &Window) -> CustomElementRegistry {
        CustomElementRegistry {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
            when_defined: DomRefCell::new(HashMap::new()),
            element_definition_is_running: Cell::new(false),
            definitions: DomRefCell::new(HashMap::new()),
        }
    }

    pub fn new(window: &Window) -> DomRoot<CustomElementRegistry> {
        reflect_dom_object(
            Box::new(CustomElementRegistry::new_inherited(window)),
            window,
        )
    }

    /// Cleans up any active promises
    /// <https://github.com/servo/servo/issues/15318>
    pub fn teardown(&self) {
        self.when_defined.borrow_mut().clear()
    }

    /// <https://html.spec.whatwg.org/multipage/#look-up-a-custom-element-definition>
    pub fn lookup_definition(
        &self,
        local_name: &LocalName,
        is: Option<&LocalName>,
    ) -> Option<Rc<CustomElementDefinition>> {
        self.definitions
            .borrow()
            .values()
            .find(|definition| {
                // Step 4-5
                definition.local_name == *local_name &&
                    (definition.name == *local_name || Some(&definition.name) == is)
            })
            .cloned()
    }

    pub fn lookup_definition_by_constructor(
        &self,
        constructor: HandleObject,
    ) -> Option<Rc<CustomElementDefinition>> {
        self.definitions
            .borrow()
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
        let global_scope = self.window.upcast::<GlobalScope>();
        unsafe {
            // Step 10.1
            if !JS_GetProperty(
                *global_scope.get_cx(),
                constructor,
                b"prototype\0".as_ptr() as *const _,
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
    /// Steps 10.3, 10.4
    #[allow(unsafe_code)]
    unsafe fn get_callbacks(&self, prototype: HandleObject) -> Fallible<LifecycleCallbacks> {
        let cx = self.window.get_cx();

        // Step 4
        Ok(LifecycleCallbacks {
            connected_callback: get_callback(cx, prototype, b"connectedCallback\0")?,
            disconnected_callback: get_callback(cx, prototype, b"disconnectedCallback\0")?,
            adopted_callback: get_callback(cx, prototype, b"adoptedCallback\0")?,
            attribute_changed_callback: get_callback(cx, prototype, b"attributeChangedCallback\0")?,
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define>
    /// Step 10.6
    #[allow(unsafe_code)]
    fn get_observed_attributes(&self, constructor: HandleObject) -> Fallible<Vec<DOMString>> {
        let cx = self.window.get_cx();
        rooted!(in(*cx) let mut observed_attributes = UndefinedValue());
        if unsafe {
            !JS_GetProperty(
                *cx,
                constructor,
                b"observedAttributes\0".as_ptr() as *const _,
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
}

/// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define>
/// Step 10.4
#[allow(unsafe_code)]
fn get_callback(
    cx: JSContext,
    prototype: HandleObject,
    name: &[u8],
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

impl CustomElementRegistryMethods for CustomElementRegistry {
    #[allow(unsafe_code, unrooted_must_root)]
    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define>
    fn Define(
        &self,
        name: DOMString,
        constructor_: Rc<CustomElementConstructor>,
        options: &ElementDefinitionOptions,
    ) -> ErrorResult {
        let cx = self.window.get_cx();
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
            .any(|(_, ref def)| def.constructor == constructor_)
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

        // Steps 10.1 - 10.2
        rooted!(in(*cx) let mut prototype = UndefinedValue());
        {
            let _ac = JSAutoRealm::new(*cx, constructor.get());
            if let Err(error) = self.check_prototype(constructor.handle(), prototype.handle_mut()) {
                self.element_definition_is_running.set(false);
                return Err(error);
            }
        };

        // Steps 10.3 - 10.4
        rooted!(in(*cx) let proto_object = prototype.to_object());
        let callbacks = {
            let _ac = JSAutoRealm::new(*cx, proto_object.get());
            match unsafe { self.get_callbacks(proto_object.handle()) } {
                Ok(callbacks) => callbacks,
                Err(error) => {
                    self.element_definition_is_running.set(false);
                    return Err(error);
                },
            }
        };

        // Step 10.5 - 10.6
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

        self.element_definition_is_running.set(false);

        // Step 11
        let definition = Rc::new(CustomElementDefinition::new(
            name.clone(),
            local_name.clone(),
            constructor_,
            observed_attributes,
            callbacks,
        ));

        // Step 12
        self.definitions
            .borrow_mut()
            .insert(name.clone(), definition.clone());

        // Step 13
        let document = self.window.Document();

        // Steps 14-15
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
                ScriptThread::enqueue_upgrade_reaction(&*candidate, definition.clone());
            }
        }

        // Step 16, 16.3
        if let Some(promise) = self.when_defined.borrow_mut().remove(&name) {
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
    fn Get(&self, cx: JSContext, name: DOMString) -> JSVal {
        match self.definitions.borrow().get(&LocalName::from(&*name)) {
            Some(definition) => unsafe {
                rooted!(in(*cx) let mut constructor = UndefinedValue());
                definition
                    .constructor
                    .to_jsval(*cx, constructor.handle_mut());
                constructor.get()
            },
            None => UndefinedValue(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-customelementregistry-whendefined>
    #[allow(unsafe_code)]
    fn WhenDefined(&self, name: DOMString, comp: InRealm) -> Rc<Promise> {
        let global_scope = self.window.upcast::<GlobalScope>();
        let name = LocalName::from(&*name);

        // Step 1
        if !is_valid_custom_element_name(&name) {
            let promise = Promise::new_in_current_realm(&global_scope, comp);
            promise.reject_native(&DOMException::new(&global_scope, DOMErrorName::SyntaxError));
            return promise;
        }

        // Step 2
        if let Some(definition) = self.definitions.borrow().get(&LocalName::from(&*name)) {
            unsafe {
                let cx = global_scope.get_cx();
                rooted!(in(*cx) let mut constructor = UndefinedValue());
                definition
                    .constructor
                    .to_jsval(*cx, constructor.handle_mut());
                let promise = Promise::new_in_current_realm(&global_scope, comp);
                promise.resolve_native(&constructor.get());
                return promise;
            }
        }

        // Step 3
        let mut map = self.when_defined.borrow_mut();

        // Steps 4, 5
        let promise = map.get(&name).cloned().unwrap_or_else(|| {
            let promise = Promise::new_in_current_realm(&global_scope, comp);
            map.insert(name, promise.clone());
            promise
        });

        // Step 6
        promise
    }
    /// https://html.spec.whatwg.org/multipage/#dom-customelementregistry-upgrade
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
pub struct LifecycleCallbacks {
    #[ignore_malloc_size_of = "Rc"]
    connected_callback: Option<Rc<Function>>,

    #[ignore_malloc_size_of = "Rc"]
    disconnected_callback: Option<Rc<Function>>,

    #[ignore_malloc_size_of = "Rc"]
    adopted_callback: Option<Rc<Function>>,

    #[ignore_malloc_size_of = "Rc"]
    attribute_changed_callback: Option<Rc<Function>>,
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum ConstructionStackEntry {
    Element(DomRoot<Element>),
    AlreadyConstructedMarker,
}

/// <https://html.spec.whatwg.org/multipage/#custom-element-definition>
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub struct CustomElementDefinition {
    pub name: LocalName,

    pub local_name: LocalName,

    #[ignore_malloc_size_of = "Rc"]
    pub constructor: Rc<CustomElementConstructor>,

    pub observed_attributes: Vec<DOMString>,

    pub callbacks: LifecycleCallbacks,

    pub construction_stack: DomRefCell<Vec<ConstructionStackEntry>>,
}

impl CustomElementDefinition {
    fn new(
        name: LocalName,
        local_name: LocalName,
        constructor: Rc<CustomElementConstructor>,
        observed_attributes: Vec<DOMString>,
        callbacks: LifecycleCallbacks,
    ) -> CustomElementDefinition {
        CustomElementDefinition {
            name: name,
            local_name: local_name,
            constructor: constructor,
            observed_attributes: observed_attributes,
            callbacks: callbacks,
            construction_stack: Default::default(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#autonomous-custom-element>
    pub fn is_autonomous(&self) -> bool {
        self.name == self.local_name
    }

    /// https://dom.spec.whatwg.org/#concept-create-element Step 6.1
    #[allow(unsafe_code)]
    pub fn create_element(
        &self,
        document: &Document,
        prefix: Option<Prefix>,
    ) -> Fallible<DomRoot<Element>> {
        let window = document.window();
        let cx = window.get_cx();
        // Step 2
        rooted!(in(*cx) let constructor = ObjectValue(self.constructor.callback()));
        rooted!(in(*cx) let mut element = ptr::null_mut::<JSObject>());
        {
            // Go into the constructor's realm
            let _ac = JSAutoRealm::new(*cx, self.constructor.callback());
            let args = HandleValueArray::new();
            if unsafe { !Construct1(*cx, constructor.handle(), &args, element.handle_mut()) } {
                return Err(Error::JSFailed);
            }
        }

        // https://heycam.github.io/webidl/#construct-a-callback-function
        // https://html.spec.whatwg.org/multipage/#clean-up-after-running-script
        if is_execution_stack_empty() {
            window
                .upcast::<GlobalScope>()
                .perform_a_microtask_checkpoint();
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

        // Step 3
        if !element.is::<HTMLElement>() {
            return Err(Error::Type(
                "Constructor did not return a DOM node".to_owned(),
            ));
        }

        // Steps 4-9
        if element.HasAttributes() ||
            element.upcast::<Node>().children_count() > 0 ||
            element.upcast::<Node>().has_parent() ||
            &*element.upcast::<Node>().owner_doc() != document ||
            *element.namespace() != ns!(html) ||
            *element.local_name() != self.local_name
        {
            return Err(Error::NotSupported);
        }

        // Step 10
        element.set_prefix(prefix);

        // Step 11
        // Element's `is` is None by default

        Ok(element)
    }
}

/// <https://html.spec.whatwg.org/multipage/#concept-upgrade-an-element>
#[allow(unsafe_code)]
pub fn upgrade_element(definition: Rc<CustomElementDefinition>, element: &Element) {
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
    let result = run_upgrade_constructor(&definition.constructor, element);

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
        let cx = global.get_cx();
        unsafe {
            let ar = enter_realm(&*global);
            throw_dom_exception(cx, &global, error);
            report_pending_exception(*cx, true, InRealm::Entered(&ar));
        }

        return;
    }

    // TODO Step 9: "If element is a form-associated custom element..."

    // Step 10
    element.set_custom_element_state(CustomElementState::Custom);
}

/// <https://html.spec.whatwg.org/multipage/#concept-upgrade-an-element>
/// Steps 8.1-8.3
#[allow(unsafe_code)]
fn run_upgrade_constructor(
    constructor: &Rc<CustomElementConstructor>,
    element: &Element,
) -> ErrorResult {
    let window = window_from_node(element);
    let cx = window.get_cx();
    rooted!(in(*cx) let constructor_val = ObjectValue(constructor.callback()));
    rooted!(in(*cx) let mut element_val = UndefinedValue());
    unsafe {
        element.to_jsval(*cx, element_val.handle_mut());
    }
    rooted!(in(*cx) let mut construct_result = ptr::null_mut::<JSObject>());
    {
        // Step 8.1 TODO when shadow DOM exists

        // Go into the constructor's realm
        let _ac = JSAutoRealm::new(*cx, constructor.callback());
        let args = HandleValueArray::new();
        // Step 8.2
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
                .upcast::<GlobalScope>()
                .perform_a_microtask_checkpoint();
        }

        // Step 8.3
        let mut same = false;
        rooted!(in(*cx) let construct_result_val = ObjectValue(construct_result.get()));
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
pub fn try_upgrade_element(element: &Element) {
    // Step 1
    let document = document_from_node(element);
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
#[unrooted_must_root_lint::must_root]
pub enum CustomElementReaction {
    Upgrade(#[ignore_malloc_size_of = "Rc"] Rc<CustomElementDefinition>),
    Callback(
        #[ignore_malloc_size_of = "Rc"] Rc<Function>,
        #[ignore_malloc_size_of = "mozjs"] Box<[Heap<JSVal>]>,
    ),
}

impl CustomElementReaction {
    /// <https://html.spec.whatwg.org/multipage/#invoke-custom-element-reactions>
    #[allow(unsafe_code)]
    pub fn invoke(&self, element: &Element) {
        // Step 2.1
        match *self {
            CustomElementReaction::Upgrade(ref definition) => {
                upgrade_element(definition.clone(), element)
            },
            CustomElementReaction::Callback(ref callback, ref arguments) => {
                // We're rooted, so it's safe to hand out a handle to objects in Heap
                let arguments = arguments
                    .iter()
                    .map(|arg| unsafe { HandleValue::from_raw(arg.handle()) })
                    .collect();
                let _ = callback.Call_(&*element, arguments, ExceptionHandling::Report);
            },
        }
    }
}

pub enum CallbackReaction {
    Connected,
    Disconnected,
    Adopted(DomRoot<Document>, DomRoot<Document>),
    AttributeChanged(LocalName, Option<DOMString>, Option<DOMString>, Namespace),
}

/// <https://html.spec.whatwg.org/multipage/#processing-the-backup-element-queue>
#[derive(Clone, Copy, Eq, JSTraceable, MallocSizeOf, PartialEq)]
enum BackupElementQueueFlag {
    Processing,
    NotProcessing,
}

/// <https://html.spec.whatwg.org/multipage/#custom-element-reactions-stack>
#[derive(JSTraceable, MallocSizeOf)]
#[unrooted_must_root_lint::must_root]
pub struct CustomElementReactionStack {
    stack: DomRefCell<Vec<ElementQueue>>,
    backup_queue: ElementQueue,
    processing_backup_element_queue: Cell<BackupElementQueueFlag>,
}

impl CustomElementReactionStack {
    pub fn new() -> CustomElementReactionStack {
        CustomElementReactionStack {
            stack: DomRefCell::new(Vec::new()),
            backup_queue: ElementQueue::new(),
            processing_backup_element_queue: Cell::new(BackupElementQueueFlag::NotProcessing),
        }
    }

    pub fn push_new_element_queue(&self) {
        self.stack.borrow_mut().push(ElementQueue::new());
    }

    pub fn pop_current_element_queue(&self) {
        rooted_vec!(let mut stack);
        mem::swap(&mut *stack, &mut *self.stack.borrow_mut());

        if let Some(current_queue) = stack.last() {
            current_queue.invoke_reactions();
        }
        stack.pop();

        mem::swap(&mut *self.stack.borrow_mut(), &mut *stack);
        self.stack.borrow_mut().append(&mut *stack);
    }

    /// <https://html.spec.whatwg.org/multipage/#enqueue-an-element-on-the-appropriate-element-queue>
    /// Step 4
    pub fn invoke_backup_element_queue(&self) {
        // Step 4.1
        self.backup_queue.invoke_reactions();

        // Step 4.2
        self.processing_backup_element_queue
            .set(BackupElementQueueFlag::NotProcessing);
    }

    /// <https://html.spec.whatwg.org/multipage/#enqueue-an-element-on-the-appropriate-element-queue>
    pub fn enqueue_element(&self, element: &Element) {
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
    pub fn enqueue_callback_reaction(
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

                let cx = element.global().get_cx();
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
    pub fn enqueue_upgrade_reaction(
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
#[unrooted_must_root_lint::must_root]
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
    fn invoke_reactions(&self) {
        // Steps 1-2
        while let Some(element) = self.next_element() {
            element.invoke_reactions()
        }
        self.queue.borrow_mut().clear();
    }

    fn next_element(&self) -> Option<DomRoot<Element>> {
        self.queue
            .borrow_mut()
            .pop_front()
            .as_ref()
            .map(Dom::deref)
            .map(DomRoot::from_ref)
    }

    fn append_element(&self, element: &Element) {
        self.queue.borrow_mut().push_back(Dom::from_ref(element));
    }
}

/// <https://html.spec.whatwg.org/multipage/#valid-custom-element-name>
pub fn is_valid_custom_element_name(name: &str) -> bool {
    // Custom elment names must match:
    // PotentialCustomElementName ::= [a-z] (PCENChar)* '-' (PCENChar)*

    let mut chars = name.chars();
    if !chars.next().map_or(false, |c| c >= 'a' && c <= 'z') {
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
        (c >= '0' && c <= '9') ||
        (c >= 'a' && c <= 'z') ||
        (c >= '\u{C0}' && c <= '\u{D6}') ||
        (c >= '\u{D8}' && c <= '\u{F6}') ||
        (c >= '\u{F8}' && c <= '\u{37D}') ||
        (c >= '\u{37F}' && c <= '\u{1FFF}') ||
        (c >= '\u{200C}' && c <= '\u{200D}') ||
        (c >= '\u{203F}' && c <= '\u{2040}') ||
        (c >= '\u{2070}' && c <= '\u{2FEF}') ||
        (c >= '\u{3001}' && c <= '\u{D7FF}') ||
        (c >= '\u{F900}' && c <= '\u{FDCF}') ||
        (c >= '\u{FDF0}' && c <= '\u{FFFD}') ||
        (c >= '\u{10000}' && c <= '\u{EFFFF}')
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
