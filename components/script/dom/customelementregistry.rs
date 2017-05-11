/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::CallbackContainer;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CustomElementRegistryBinding;
use dom::bindings::codegen::Bindings::CustomElementRegistryBinding::CustomElementRegistryMethods;
use dom::bindings::codegen::Bindings::CustomElementRegistryBinding::ElementDefinitionOptions;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::window::Window;
use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{IsConstructor, HandleObject, JS_GetProperty, JSAutoCompartment, JSContext};
use js::jsval::{JSVal, UndefinedValue};
use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::CString;
use std::rc::Rc;

// https://html.spec.whatwg.org/multipage/#customelementregistry
#[dom_struct]
pub struct CustomElementRegistry {
    reflector_: Reflector,

    window: JS<Window>,

    #[ignore_heap_size_of = "Rc"]
    when_defined: DOMRefCell<HashMap<DOMString, Rc<Promise>>>,

    element_definition_is_running: Cell<bool>,

    definitions: DOMRefCell<HashMap<DOMString, CustomElementDefinition>>,
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

    // Cleans up any active promises
    // https://github.com/servo/servo/issues/15318
    pub fn teardown(&self) {
        self.when_defined.borrow_mut().clear()
    }

    // https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define
    // Steps 10.1, 10.2
    #[allow(unsafe_code)]
    fn check_prototype(&self, constructor: HandleObject) -> ErrorResult {
        let global_scope = self.window.upcast::<GlobalScope>();
        rooted!(in(global_scope.get_cx()) let mut prototype = UndefinedValue());
        unsafe {
            // Step 10.1
            let c_name = CString::new("prototype").unwrap();
            if !JS_GetProperty(global_scope.get_cx(), constructor, c_name.as_ptr(), prototype.handle_mut()) {
                return Err(Error::JSFailed);
            }

            // Step 10.2
            if !prototype.is_object() {
                return Err(Error::Type("constructor.protoype is not an object".to_owned()));
            }
        }
        Ok(())
    }
}

impl CustomElementRegistryMethods for CustomElementRegistry {
    #[allow(unsafe_code, unrooted_must_root)]
    // https://html.spec.whatwg.org/multipage/#dom-customelementregistry-define
    fn Define(&self, name: DOMString, constructor_: Rc<Function>, options: &ElementDefinitionOptions) -> ErrorResult {
        let global_scope = self.window.upcast::<GlobalScope>();
        rooted!(in(global_scope.get_cx()) let constructor = constructor_.callback());

        // Step 1
        if unsafe { !IsConstructor(constructor.get()) } {
            return Err(Error::Type("Second argument of CustomElementRegistry.defin is not a constructor".to_owned()));
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
        let local_name = match *extends {
            Some(ref extended_name) => {
                // Step 7.1
                if is_valid_custom_element_name(extended_name) {
                    return Err(Error::NotSupported)
                }

                // TODO: 7.2
                // Check that the element interface for extends is not HTMLUnknownElement

                extended_name.clone()
            },
            // Step 7.3
            None => name.clone(),
        };

        // Step 8
        if self.element_definition_is_running.get() {
            return Err(Error::NotSupported);
        }

        // Step 9
        self.element_definition_is_running.set(true);

        // Steps 10.1 - 10.2
        let result = {
            let _ac = JSAutoCompartment::new(global_scope.get_cx(), constructor.get());
            self.check_prototype(constructor.handle())
        };

        // TODO: Steps 10.3 - 10.6
        // 10.3 - 10.4 Handle lifecycle callbacks
        // 10.5 - 10.6 Get observed attributes from the constructor

        self.element_definition_is_running.set(false);
        if let Err(error) = result {
            return Err(error);
        }

        // Step 11
        let definition = CustomElementDefinition::new(name.clone(), local_name, constructor_);

        // Step 12
        self.definitions.borrow_mut().insert(name.clone(), definition);

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

    // https://html.spec.whatwg.org/multipage/#dom-customelementregistry-get
    #[allow(unsafe_code)]
    unsafe fn Get(&self, cx: *mut JSContext, name: DOMString) -> JSVal {
        match self.definitions.borrow_mut().get(&name) {
            Some(definition) => {
                rooted!(in(cx) let mut constructor = UndefinedValue());
                definition.constructor.to_jsval(cx, constructor.handle_mut());
                constructor.get()
            },
            None => UndefinedValue(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-customelementregistry-whendefined
    #[allow(unrooted_must_root)]
    fn WhenDefined(&self, name: DOMString) -> Fallible<Rc<Promise>> {
        let global_scope = self.window.upcast::<GlobalScope>();

        // Step 1
        if !is_valid_custom_element_name(&name) {
            return Err(Error::Syntax);
        }

        // Step 2
        if self.definitions.borrow().contains_key(&name) {
            let promise = Promise::new(global_scope);
            promise.resolve_native(global_scope.get_cx(), &UndefinedValue());
            return Ok(promise)
        }

        // Step 3
        let mut map = self.when_defined.borrow_mut();

        // Steps 4, 5
        let promise = match map.get(&name).cloned() {
            Some(promise) => promise,
            None => {
                let promise = Promise::new(global_scope);
                map.insert(name, promise.clone());
                promise
            }
        };

        // Step 6
        Ok(promise)
    }
}

#[derive(HeapSizeOf, JSTraceable)]
struct CustomElementDefinition {
    name: DOMString,

    local_name: DOMString,

    #[ignore_heap_size_of = "Rc"]
    constructor: Rc<Function>,
}

impl CustomElementDefinition {
    fn new(name: DOMString, local_name: DOMString, constructor: Rc<Function>) -> CustomElementDefinition {
        CustomElementDefinition {
            name: name,
            local_name: local_name,
            constructor: constructor,
        }
    }
}

// https://html.spec.whatwg.org/multipage/#valid-custom-element-name
fn is_valid_custom_element_name(name: &str) -> bool {
    // Custom elment names must match:
    // PotentialCustomElementName ::= [a-z] (PCENChar)* '-' (PCENChar)*

    if let Some(c) = name.chars().nth(0) {
        if c < 'a' || c > 'z' {
            return false;
        }
    } else {
        return false;
    }

    let mut has_dash = false;

    for c in name.chars() {
        if c == '-' {
            has_dash = true;
        }

        // Check if this character is not a PCENChar
        // https://html.spec.whatwg.org/multipage/#prod-pcenchar
        if c != '-' && c != '.' && c != '_' && c != '\u{B7}' &&
            (c < '0' || c > '9') &&
            (c < 'a' || c > 'z') &&
            (c < '\u{C0}' || c > '\u{D6}') &&
            (c < '\u{D8}' || c > '\u{F6}') &&
            (c < '\u{F8}' || c > '\u{37D}') &&
            (c < '\u{37F}' || c > '\u{1FFF}') &&
            (c < '\u{200C}' || c > '\u{200D}') &&
            (c < '\u{203F}' || c > '\u{2040}') &&
            (c < '\u{2070}' || c > '\u{2FEF}') &&
            (c < '\u{3001}' || c > '\u{D7FF}') &&
            (c < '\u{F900}' || c > '\u{FDCF}') &&
            (c < '\u{FDF0}' || c > '\u{FFFD}') &&
            (c < '\u{10000}' || c > '\u{EFFFF}')
        {
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
