/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::{CallbackContainer, ExceptionHandling};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CustomElementRegistryBinding;
use dom::bindings::codegen::Bindings::CustomElementRegistryBinding::CustomElementRegistryMethods;
use dom::bindings::codegen::Bindings::CustomElementRegistryBinding::ElementDefinitionOptions;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::conversions::{ConversionResult, FromJSValConvertible, StringificationBehavior};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::domexception::{DOMErrorName, DOMException};
use dom::element::Element;
use dom::globalscope::GlobalScope;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::promise::Promise;
use dom::window::Window;
use dom_struct::dom_struct;
use html5ever::{LocalName, Namespace, Prefix};
use js::conversions::ToJSValConvertible;
use js::jsapi::{Construct1, IsCallable, IsConstructor, HandleValueArray, HandleObject, MutableHandleValue};
use js::jsapi::{Heap, JS_GetProperty, JSAutoCompartment, JSContext};
use js::jsval::{JSVal, NullValue, ObjectValue, UndefinedValue};
use microtask::Microtask;
use script_thread::ScriptThread;
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

/// https://html.spec.whatwg.org/multipage/#customelementregistry
#[dom_struct]
pub struct CustomElementRegistry {
    reflector_: Reflector,

    window: JS<Window>,

    #[ignore_heap_size_of = "Rc"]
    when_defined: DOMRefCell<HashMap<LocalName, Rc<Promise>>>,

    element_definition_is_running: Cell<bool>,

    #[ignore_heap_size_of = "Rc"]
    definitions: DOMRefCell<HashMap<LocalName, Rc<CustomElementDefinition>>>,
}

impl CustomElementRegistry {
    fn new_inherited(window: &Window) -> CustomElementRegistry {
        CustomElementRegistry {
            reflector_: Reflector::new(),
            window: JS::from_ref(window),
            when_defined: DOMRefCell::new(HashMap::new()),
            element_definition_is_running: Cell::new(false),
            definitions: DOMRefCell::new(HashMap::new()),
        }
    }

    pub fn new(window: &Window) -> Root<CustomElementRegistry> {
        reflect_dom_object(box CustomElementRegistry::new_inherited(window),
                           window,
                           CustomElementRegistryBinding::Wrap)
    }

    /// Cleans up any active promises
    /// https://github.com/servo/servo/issues/15318
    pub fn teardown(&self) {
        self.when_defined.borrow_mut().clear()
    }

    /// https://html.spec.whatwg.org/multipage/#look-up-a-custom-element-definition
    pub fn lookup_definition(&self,
                             local_name: LocalName,
                             is: Option<LocalName>)
                             -> Option<Rc<CustomElementDefinition>> {
        self.definitions.borrow().values().find(|definition| {
            // Step 4-5
            definition.local_name == local_name &&
                (definition.name == local_name || Some(&definition.name) == is.as_ref())
        }).cloned()
    }

    pub fn lookup_definition_by_constructor(&self, constructor: HandleObject) -> Option<Rc<CustomElementDefinition>> {
        self.definitions.borrow().values().find(|definition| {
            definition.constructor.callback() == constructor.get()
        }).cloned()
    }

    /// https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define
    /// Steps 10.1, 10.2
    #[allow(unsafe_code)]
    fn check_prototype(&self, constructor: HandleObject, prototype: MutableHandleValue) -> ErrorResult {
        let global_scope = self.window.upcast::<GlobalScope>();
        unsafe {
            // Step 10.1
            if !JS_GetProperty(global_scope.get_cx(),
                               constructor,
                               b"prototype\0".as_ptr() as *const _,
                               prototype) {
                return Err(Error::JSFailed);
            }

            // Step 10.2
            if !prototype.is_object() {
                return Err(Error::Type("constructor.prototype is not an object".to_owned()));
            }
        }
        Ok(())
    }

    /// https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define
    /// Steps 10.3, 10.4
    fn get_callbacks(&self, prototype: HandleObject) -> Fallible<LifecycleCallbacks> {
        let cx = self.window.get_cx();

        // Step 4
        Ok(LifecycleCallbacks {
            connected_callback: get_callback(cx, prototype, b"connectedCallback\0")?,
            disconnected_callback: get_callback(cx, prototype, b"disconnectedCallback\0")?,
            adopted_callback: get_callback(cx, prototype, b"adoptedCallback\0")?,
            attribute_changed_callback: get_callback(cx, prototype, b"attributeChangedCallback\0")?,
        })
    }

    /// https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define
    /// Step 10.6
    #[allow(unsafe_code)]
    fn get_observed_attributes(&self, constructor: HandleObject) -> Fallible<Vec<DOMString>> {
        let cx = self.window.get_cx();
        rooted!(in(cx) let mut observed_attributes = UndefinedValue());
        if unsafe { !JS_GetProperty(cx,
                                    constructor,
                                    b"observedAttributes\0".as_ptr() as *const _,
                                    observed_attributes.handle_mut()) } {
            return Err(Error::JSFailed);
        }

        if observed_attributes.is_undefined() {
            return Ok(Vec::new());
        }

        let conversion = unsafe {
            FromJSValConvertible::from_jsval(cx, observed_attributes.handle(), StringificationBehavior::Default)
        };
        match conversion {
            Ok(ConversionResult::Success(attributes)) => Ok(attributes),
            Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.into())),
            _ => Err(Error::JSFailed),
        }
    }
}

/// https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define
/// Step 10.4
#[allow(unsafe_code)]
fn get_callback(cx: *mut JSContext, prototype: HandleObject, name: &[u8]) -> Fallible<Option<Rc<Function>>> {
    rooted!(in(cx) let mut callback = UndefinedValue());

    // Step 10.4.1
    if unsafe { !JS_GetProperty(cx,
                                prototype,
                                name.as_ptr() as *const _,
                                callback.handle_mut()) } {
        return Err(Error::JSFailed);
    }

    // Step 10.4.2
    if !callback.is_undefined() {
        if !callback.is_object() || unsafe { !IsCallable(callback.to_object()) } {
            return Err(Error::Type("Lifecycle callback is not callable".to_owned()));
        }
        Ok(Some(Function::new(cx, callback.to_object())))
    } else {
        Ok(None)
    }
}

impl CustomElementRegistryMethods for CustomElementRegistry {
    #[allow(unsafe_code, unrooted_must_root)]
    /// https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define
    fn Define(&self, name: DOMString, constructor_: Rc<Function>, options: &ElementDefinitionOptions) -> ErrorResult {
        let cx = self.window.get_cx();
        rooted!(in(cx) let constructor = constructor_.callback());
        let name = LocalName::from(&*name);

        // Step 1
        if unsafe { !IsConstructor(constructor.get()) } {
            return Err(Error::Type("Second argument of CustomElementRegistry.define is not a constructor".to_owned()));
        }

        // Step 2
        if !is_valid_custom_element_name(&name) {
            return Err(Error::Syntax)
        }

        // Step 3
        if self.definitions.borrow().contains_key(&name) {
            return Err(Error::NotSupported);
        }

        // Step 4
        if self.definitions.borrow().iter().any(|(_, ref def)| def.constructor == constructor_) {
            return Err(Error::NotSupported);
        }

        // Step 6
        let extends = &options.extends;

        // Steps 5, 7
        let local_name = if let Some(ref extended_name) = *extends {
            // Step 7.1
            if is_valid_custom_element_name(extended_name) {
                return Err(Error::NotSupported)
            }

            // Step 7.2
            if !is_extendable_element_interface(extended_name) {
                return Err(Error::NotSupported)
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
        rooted!(in(cx) let mut prototype = UndefinedValue());
        {
            let _ac = JSAutoCompartment::new(cx, constructor.get());
            if let Err(error) = self.check_prototype(constructor.handle(), prototype.handle_mut()) {
                self.element_definition_is_running.set(false);
                return Err(error);
            }
        };

        // Steps 10.3 - 10.4
        rooted!(in(cx) let proto_object = prototype.to_object());
        let callbacks = {
            let _ac = JSAutoCompartment::new(cx, proto_object.get());
            match self.get_callbacks(proto_object.handle()) {
                Ok(callbacks) => callbacks,
                Err(error) => {
                    self.element_definition_is_running.set(false);
                    return Err(error);
                },
            }
        };

        // Step 10.5 - 10.6
        let observed_attributes = if callbacks.attribute_changed_callback.is_some() {
            let _ac = JSAutoCompartment::new(cx, constructor.get());
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
        let definition = CustomElementDefinition::new(name.clone(),
                                                      local_name,
                                                      constructor_,
                                                      observed_attributes,
                                                      callbacks);

        // Step 12
        self.definitions.borrow_mut().insert(name.clone(), Rc::new(definition));

        // TODO: Step 13, 14, 15
        // Handle custom element upgrades

        // Step 16, 16.3
        if let Some(promise) = self.when_defined.borrow_mut().remove(&name) {
            // 16.1
            let cx = promise.global().get_cx();
            // 16.2
            promise.resolve_native(cx, &UndefinedValue());
        }
        Ok(())
    }

    /// https://html.spec.whatwg.org/multipage/#dom-customelementregistry-get
    #[allow(unsafe_code)]
    unsafe fn Get(&self, cx: *mut JSContext, name: DOMString) -> JSVal {
        match self.definitions.borrow().get(&LocalName::from(&*name)) {
            Some(definition) => {
                rooted!(in(cx) let mut constructor = UndefinedValue());
                definition.constructor.to_jsval(cx, constructor.handle_mut());
                constructor.get()
            },
            None => UndefinedValue(),
        }
    }

    /// https://html.spec.whatwg.org/multipage/#dom-customelementregistry-whendefined
    #[allow(unrooted_must_root)]
    fn WhenDefined(&self, name: DOMString) -> Rc<Promise> {
        let global_scope = self.window.upcast::<GlobalScope>();
        let name = LocalName::from(&*name);

        // Step 1
        if !is_valid_custom_element_name(&name) {
            let promise = Promise::new(global_scope);
            promise.reject_native(global_scope.get_cx(), &DOMException::new(global_scope, DOMErrorName::SyntaxError));
            return promise
        }

        // Step 2
        if self.definitions.borrow().contains_key(&name) {
            let promise = Promise::new(global_scope);
            promise.resolve_native(global_scope.get_cx(), &UndefinedValue());
            return promise
        }

        // Step 3
        let mut map = self.when_defined.borrow_mut();

        // Steps 4, 5
        let promise = map.get(&name).cloned().unwrap_or_else(|| {
            let promise = Promise::new(global_scope);
            map.insert(name, promise.clone());
            promise
        });

        // Step 6
        promise
    }
}

#[derive(HeapSizeOf, JSTraceable, Clone)]
pub struct LifecycleCallbacks {
    #[ignore_heap_size_of = "Rc"]
    connected_callback: Option<Rc<Function>>,

    #[ignore_heap_size_of = "Rc"]
    disconnected_callback: Option<Rc<Function>>,

    #[ignore_heap_size_of = "Rc"]
    adopted_callback: Option<Rc<Function>>,

    #[ignore_heap_size_of = "Rc"]
    attribute_changed_callback: Option<Rc<Function>>,
}

/// https://html.spec.whatwg.org/multipage/#custom-element-definition
#[derive(HeapSizeOf, JSTraceable, Clone)]
pub struct CustomElementDefinition {
    pub name: LocalName,

    pub local_name: LocalName,

    #[ignore_heap_size_of = "Rc"]
    pub constructor: Rc<Function>,

    pub observed_attributes: Vec<DOMString>,

    pub callbacks: LifecycleCallbacks,
}

impl CustomElementDefinition {
    fn new(name: LocalName,
           local_name: LocalName,
           constructor: Rc<Function>,
           observed_attributes: Vec<DOMString>,
           callbacks: LifecycleCallbacks)
           -> CustomElementDefinition {
        CustomElementDefinition {
            name: name,
            local_name: local_name,
            constructor: constructor,
            observed_attributes: observed_attributes,
            callbacks: callbacks,
        }
    }

    /// https://html.spec.whatwg.org/multipage/#autonomous-custom-element
    pub fn is_autonomous(&self) -> bool {
        self.name == self.local_name
    }

    /// https://dom.spec.whatwg.org/#concept-create-element Step 6.1
    #[allow(unsafe_code)]
    pub fn create_element(&self, document: &Document, prefix: Option<Prefix>) -> Fallible<Root<Element>> {
        let window = document.window();
        let cx = window.get_cx();
        // Step 2
        rooted!(in(cx) let constructor = ObjectValue(self.constructor.callback()));
        rooted!(in(cx) let mut element = ptr::null_mut());
        {
            // Go into the constructor's compartment
            let _ac = JSAutoCompartment::new(cx, self.constructor.callback());
            let args = HandleValueArray::new();
            if unsafe { !Construct1(cx, constructor.handle(), &args, element.handle_mut()) } {
                return Err(Error::JSFailed);
            }
        }

        rooted!(in(cx) let element_val = ObjectValue(element.get()));
        let element: Root<Element> = match unsafe { Root::from_jsval(cx, element_val.handle(), ()) } {
            Ok(ConversionResult::Success(element)) => element,
            Ok(ConversionResult::Failure(..)) =>
                return Err(Error::Type("Constructor did not return a DOM node".to_owned())),
            _ => return Err(Error::JSFailed),
        };

        // Step 3
        if !element.is::<HTMLElement>() {
            return Err(Error::Type("Constructor did not return a DOM node".to_owned()));
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

#[derive(HeapSizeOf, JSTraceable)]
#[must_root]
pub enum CustomElementReaction {
    // TODO: Support upgrade reactions
    Callback(
        #[ignore_heap_size_of = "Rc"]
        Rc<Function>,
        Box<[Heap<JSVal>]>
    ),
}

impl CustomElementReaction {
    /// https://html.spec.whatwg.org/multipage/#invoke-custom-element-reactions
    #[allow(unsafe_code)]
    pub fn invoke(&self, element: &Element) {
        // Step 2.1
        match *self {
            CustomElementReaction::Callback(ref callback, ref arguments) => {
                let arguments = arguments.iter().map(|arg| arg.handle()).collect();
                let _ = callback.Call_(&*element, arguments, ExceptionHandling::Report);
            }
        }
    }
}

pub enum CallbackReaction {
    Connected,
    Disconnected,
    Adopted(Root<Document>, Root<Document>),
    AttributeChanged(LocalName, Option<DOMString>, Option<DOMString>, Namespace),
}

/// https://html.spec.whatwg.org/multipage/#processing-the-backup-element-queue
#[derive(HeapSizeOf, JSTraceable, Eq, PartialEq, Clone, Copy)]
enum BackupElementQueueFlag {
    Processing,
    NotProcessing,
}

/// https://html.spec.whatwg.org/multipage/#custom-element-reactions-stack
#[derive(HeapSizeOf, JSTraceable)]
#[must_root]
pub struct CustomElementReactionStack {
    backup_queue: ElementQueue,
    processing_backup_element_queue: Cell<BackupElementQueueFlag>,
}

impl CustomElementReactionStack {
    pub fn new() -> CustomElementReactionStack {
        CustomElementReactionStack {
            backup_queue: ElementQueue::new(),
            processing_backup_element_queue: Cell::new(BackupElementQueueFlag::NotProcessing),
        }
    }

    /// https://html.spec.whatwg.org/multipage/#enqueue-an-element-on-the-appropriate-element-queue
    /// Step 4
    pub fn invoke_backup_element_queue(&self) {
        // Step 4.1
        self.backup_queue.invoke_reactions();

        // Step 4.2
        self.processing_backup_element_queue.set(BackupElementQueueFlag::NotProcessing);
    }

    /// https://html.spec.whatwg.org/multipage/#enqueue-an-element-on-the-appropriate-element-queue
    pub fn enqueue_element(&self, element: &Element) {
        // TODO: Steps 1 - 2
        // Support multiple queues

        // Step 1.1
        self.backup_queue.append_element(element);

        // Step 1.2
        if self.processing_backup_element_queue.get() == BackupElementQueueFlag::Processing {
            return;
        }

        // Step 1.3
        self.processing_backup_element_queue.set(BackupElementQueueFlag::Processing);

        // Step 4
        ScriptThread::enqueue_microtask(Microtask::CustomElementReaction);
    }

    /// https://html.spec.whatwg.org/multipage/#enqueue-a-custom-element-callback-reaction
    #[allow(unsafe_code)]
    pub fn enqueue_callback_reaction(&self, element: &Element, reaction: CallbackReaction) {
        // Step 1
        let definition = match element.get_custom_element_definition() {
            Some(definition) => definition,
            None => return,
        };

        // Step 2
        let (callback, args) = match reaction {
            CallbackReaction::Connected => (definition.callbacks.connected_callback.clone(), Vec::new()),
            CallbackReaction::Disconnected => (definition.callbacks.disconnected_callback.clone(), Vec::new()),
            CallbackReaction::Adopted(ref old_doc, ref new_doc) => {
                let args = vec![Heap::default(), Heap::default()];
                args[0].set(ObjectValue(old_doc.reflector().get_jsobject().get()));
                args[1].set(ObjectValue(new_doc.reflector().get_jsobject().get()));
                (definition.callbacks.adopted_callback.clone(), args)
            },
            CallbackReaction::AttributeChanged(local_name, old_val, val, namespace) => {
                // Step 4
                if !definition.observed_attributes.iter().any(|attr| *attr == *local_name) {
                    return;
                }

                let cx = element.global().get_cx();

                let local_name = DOMString::from(&*local_name);
                rooted!(in(cx) let mut name_value = UndefinedValue());
                unsafe { local_name.to_jsval(cx, name_value.handle_mut()); }

                rooted!(in(cx) let mut old_value = NullValue());
                if let Some(old_val) = old_val {
                    unsafe { old_val.to_jsval(cx, old_value.handle_mut()); }
                }

                rooted!(in(cx) let mut value = NullValue());
                if let Some(val) = val {
                    unsafe { val.to_jsval(cx, value.handle_mut()); }
                }

                let namespace = DOMString::from(&*namespace);
                rooted!(in(cx) let mut namespace_value = UndefinedValue());
                unsafe { namespace.to_jsval(cx, namespace_value.handle_mut()); }

                let args = vec![Heap::default(), Heap::default(), Heap::default(), Heap::default()];
                args[0].set(name_value.get());
                args[1].set(old_value.get());
                args[2].set(value.get());
                args[3].set(namespace_value.get());

                (definition.callbacks.attribute_changed_callback.clone(), args)
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
}

/// https://html.spec.whatwg.org/multipage/#element-queue
#[derive(HeapSizeOf, JSTraceable)]
#[must_root]
struct ElementQueue {
    queue: DOMRefCell<VecDeque<JS<Element>>>,
}

impl ElementQueue {
    fn new() -> ElementQueue {
        ElementQueue {
            queue: Default::default(),
        }
    }

    /// https://html.spec.whatwg.org/multipage/#invoke-custom-element-reactions
    fn invoke_reactions(&self) {
        // Steps 1-2
        while let Some(element) = self.next_element() {
            element.invoke_reactions()
        }
        self.queue.borrow_mut().clear();
    }

    fn next_element(&self) -> Option<Root<Element>> {
        self.queue.borrow_mut().pop_front().as_ref().map(JS::deref).map(Root::from_ref)
    }

    fn append_element(&self, element: &Element) {
        self.queue.borrow_mut().push_back(JS::from_ref(element));
    }
}

/// https://html.spec.whatwg.org/multipage/#valid-custom-element-name
fn is_valid_custom_element_name(name: &str) -> bool {
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
/// https://html.spec.whatwg.org/multipage/#prod-pcenchar
fn is_potential_custom_element_char(c: char) -> bool {
    c == '-' || c == '.' || c == '_' || c == '\u{B7}' ||
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
