/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::cell::RefCell;

use js::context::JSContext;
use script_bindings::codegen::GenericUnionTypes::{
    StringOrElementCreationOptions, TrustedHTMLOrTrustedScriptOrTrustedScriptURLOrString,
};
use script_bindings::realms::enter_auto_realm;
use wasmtime::component::{Component, HasSelf, Linker, Resource};
use wasmtime::{Config, Engine, Store};

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLInputElementBinding::{
    HTMLInputElement, HTMLInputElementMethods,
};
use crate::dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::{
    HTMLTextAreaElement, HTMLTextAreaElementMethods,
};
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::Node;

// 1. Generate Rust types from the WIT definition using Wasmtime 49 bindgen
pub mod wit_dom {
    wasmtime::component::bindgen!({
        world: "app",
        path: "wit/servo_dom.wit",
    });
}

// Import the generated host traits and world
use wit_dom::App;
use wit_dom::servo::dom::console::Host as ConsoleHost;
use wit_dom::servo::dom::document::{Host as DocumentHost, HostElement};

// Thread-local storage to keep active Wasm components alive for events on the ScriptThread
thread_local! {
    pub static ACTIVE_WASM_APPS: RefCell<Vec<Option<(Store<WasmHostState>, App)>>> = RefCell::new(Vec::new());
}

/// Global dispatcher called when a DOM event fires for a Wasm listener
pub fn dispatch_wasm_event(
    cx: &mut JSContext,
    app_index: usize,
    handler_id: &str,
    event_type: &str,
) {
    // 1. Temporarily take the (Store, App) out of TLS during the call to prevent TLS re-entrancy issues
    let instance = ACTIVE_WASM_APPS.with(|apps| {
        let mut apps_borrow = apps.borrow_mut();
        if app_index < apps_borrow.len() {
            apps_borrow[app_index].take()
        } else {
            None
        }
    });

    if let Some((mut store, app)) = instance {
        // 2. Refresh the ambient JSContext in WasmHostState with the active context for this event turn
        store.data_mut().cx = cx;

        // 3. Invoke Wasm guest on_event
        if let Err(e) = app.call_on_event(&mut store, handler_id, event_type) {
            eprintln!("[Servo Wasm Error on-event]: {:?}", e);
        }

        // 4. Put the instance back into TLS
        ACTIVE_WASM_APPS.with(|apps| {
            let mut apps_borrow = apps.borrow_mut();
            if app_index < apps_borrow.len() {
                apps_borrow[app_index] = Some((store, app));
            }
        });
    }
}

/// The Host state held inside the Wasmtime Store for each component invocation
pub struct WasmHostState {
    pub cx: *mut JSContext,
    pub global_scope: DomRoot<GlobalScope>,
    pub elements: Vec<Option<DomRoot<Element>>>,
}

impl WasmHostState {
    pub fn new(cx: &mut JSContext, global_scope: DomRoot<GlobalScope>) -> Self {
        Self {
            cx: cx as *mut JSContext,
            global_scope,
            elements: Vec::new(),
        }
    }
}

// 2. Implement the Console host API
impl ConsoleHost for WasmHostState {
    fn log(&mut self, message: String) -> () {
        println!("[Servo Wasm Console.log]: {}", message);
    }

    fn error(&mut self, message: String) -> () {
        eprintln!("[Servo Wasm Console.error]: {}", message);
    }
}

// 3. Implement the Element resource lifecycle trait (HostElement)
impl HostElement for WasmHostState {
    fn add_event_listener(
        &mut self,
        self_rep: Resource<wit_dom::servo::dom::document::Element>,
        event_type: String,
        handler_id: String,
    ) -> () {
        let id = self_rep.rep() as usize;
        if let Some(Some(element)) = self.elements.get(id) {
            let event_target = element.upcast::<crate::dom::eventtarget::EventTarget>();
            event_target.add_wasm_event_listener(
                DOMString::from(event_type.clone()),
                DOMString::from(handler_id.clone()),
            );
            println!(
                "[Servo Wasm Host]: Registered native Wasm listener on EventTarget for '{}' with handler_id '{}'",
                event_type, handler_id
            );
        }
    }

    fn remove_event_listener(
        &mut self,
        self_rep: Resource<wit_dom::servo::dom::document::Element>,
        _event_type: String,
        _handler_id: String,
    ) -> () {
        let _id = self_rep.rep() as usize;
        // In the future, match and remove from EventTarget listeners
    }

    fn drop(
        &mut self,
        rep: Resource<wit_dom::servo::dom::document::Element>,
    ) -> wasmtime::Result<()> {
        let id = rep.rep() as usize;
        if id < self.elements.len() {
            self.elements[id] = None;
        }
        Ok(())
    }

    fn set_attribute(
        &mut self,
        self_rep: Resource<wit_dom::servo::dom::document::Element>,
        name: String,
        value: String,
    ) -> () {
        let id = self_rep.rep() as usize;
        if let Some(Some(element)) = self.elements.get(id) {
            let cx = unsafe { &mut *self.cx };
            let val = TrustedHTMLOrTrustedScriptOrTrustedScriptURLOrString::String(
                DOMString::from(value),
            );
            let _ = element.SetAttribute(cx, DOMString::from(name), val);
        }
    }

    fn get_attribute(
        &mut self,
        self_rep: Resource<wit_dom::servo::dom::document::Element>,
        name: String,
    ) -> Option<String> {
        let id = self_rep.rep() as usize;
        let cx = unsafe { &mut *self.cx };
        if let Some(Some(element)) = self.elements.get(id) {
            element
                .GetAttribute(cx, DOMString::from(name))
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    fn get_property(
        &mut self,
        self_rep: Resource<wit_dom::servo::dom::document::Element>,
        name: String,
    ) -> String {
        let id = self_rep.rep() as usize;
        if let Some(Some(element)) = self.elements.get(id) {
            let cx = unsafe { &mut *self.cx };
            // let global = self.global_scope;
            // let _realm = enter_auto_realm(cx, &*global);

            if name == "value" {
                if let Some(input) = element.downcast::<HTMLInputElement>() {
                    return input.Value().to_string();
                } else if let Some(textarea) = element.downcast::<HTMLTextAreaElement>() {
                    return textarea.Value().to_string();
                }
            }
        }
        String::new()
    }

    fn set_property(
        &mut self,
        self_rep: Resource<wit_dom::servo::dom::document::Element>,
        name: String,
        value: String,
    ) -> () {
        let id = self_rep.rep() as usize;
        if let Some(Some(element)) = self.elements.get(id) {
            let cx = unsafe { &mut *self.cx };
            // let global = self.global_scope;
            // let _realm = enter_auto_realm(cx, &*global);

            if name == "value" {
                if let Some(input) = element.downcast::<HTMLInputElement>() {
                    input.SetValue(cx, DOMString::from(value));
                } else if let Some(textarea) = element.downcast::<HTMLTextAreaElement>() {
                    textarea.SetValue(cx, DOMString::from(value));
                }
            }
        }
    }

    fn append_child(
        &mut self,
        self_rep: Resource<wit_dom::servo::dom::document::Element>,
        child_rep: Resource<wit_dom::servo::dom::document::Element>,
    ) -> () {
        let parent_id = self_rep.rep() as usize;
        let child_id = child_rep.rep() as usize;

        let parent = self
            .elements
            .get(parent_id)
            .and_then(|e| e.as_ref())
            .cloned();
        let child = self
            .elements
            .get(child_id)
            .and_then(|e| e.as_ref())
            .cloned();

        if let (Some(parent), Some(child)) = (parent, child) {
            let cx = unsafe { &mut *self.cx };
            let parent_node = parent.upcast::<Node>();
            let child_node = child.upcast::<Node>();
            let _ = parent_node.AppendChild(cx, child_node);
        }
    }

    fn insert_before(
        &mut self,
        self_rep: Resource<wit_dom::servo::dom::document::Element>,
        child_rep: Resource<wit_dom::servo::dom::document::Element>,
        ref_child_rep: Resource<wit_dom::servo::dom::document::Element>,
    ) -> () {
        let parent_id = self_rep.rep() as usize;
        let child_id = child_rep.rep() as usize;
        let ref_id = ref_child_rep.rep() as usize;

        let parent = self
            .elements
            .get(parent_id)
            .and_then(|e| e.as_ref())
            .cloned();
        let child = self
            .elements
            .get(child_id)
            .and_then(|e| e.as_ref())
            .cloned();
        let ref_child = self.elements.get(ref_id).and_then(|e| e.as_ref()).cloned();

        if let (Some(parent), Some(child)) = (parent, child) {
            let cx = unsafe { &mut *self.cx };
            let parent_node = parent.upcast::<Node>();
            let child_node = child.upcast::<Node>();
            let ref_node = ref_child.as_ref().map(|r| r.upcast::<Node>());
            let _ = parent_node.InsertBefore(cx, child_node, ref_node);
        }
    }

    fn remove_child(
        &mut self,
        self_rep: Resource<wit_dom::servo::dom::document::Element>,
        child_rep: Resource<wit_dom::servo::dom::document::Element>,
    ) -> () {
        let parent_id = self_rep.rep() as usize;
        let child_id = child_rep.rep() as usize;

        let parent = self
            .elements
            .get(parent_id)
            .and_then(|e| e.as_ref())
            .cloned();
        let child = self
            .elements
            .get(child_id)
            .and_then(|e| e.as_ref())
            .cloned();

        if let (Some(parent), Some(child)) = (parent, child) {
            let cx = unsafe { &mut *self.cx };
            let parent_node = parent.upcast::<Node>();
            let child_node = child.upcast::<Node>();
            let _ = parent_node.RemoveChild(cx, child_node);
        }
    }

    fn set_text_content(
        &mut self,
        self_rep: Resource<wit_dom::servo::dom::document::Element>,
        text: String,
    ) -> () {
        let id = self_rep.rep() as usize;
        if let Some(Some(element)) = self.elements.get(id) {
            let cx = unsafe { &mut *self.cx };
            let node = element.upcast::<Node>();
            let _ = node.SetTextContent(cx, Some(DOMString::from(text)));
        }
    }

    fn get_text_content(
        &mut self,
        self_rep: Resource<wit_dom::servo::dom::document::Element>,
    ) -> String {
        let id = self_rep.rep() as usize;
        if let Some(Some(element)) = self.elements.get(id) {
            let node = element.upcast::<Node>();
            node.GetTextContent()
                .map(|s| s.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    }
}

// 4. Implement the Document host API
impl DocumentHost for WasmHostState {
    fn create_element(
        &mut self,
        tag: String,
    ) -> Result<Resource<wit_dom::servo::dom::document::Element>, String> {
        let window = self.global_scope.as_window();
        let doc = window.Document();
        let dom_tag = DOMString::from(tag);
        let options = StringOrElementCreationOptions::String(DOMString::new());
        let cx = unsafe { &mut *self.cx };

        match doc.CreateElement(cx, dom_tag, options) {
            Ok(element) => {
                let id = self.elements.len() as u32;
                self.elements.push(Some(element));
                Ok(Resource::new_own(id))
            },
            Err(_) => Err("Failed to create DOM element".to_string()),
        }
    }

    fn get_body(&mut self) -> Result<Resource<wit_dom::servo::dom::document::Element>, String> {
        let window = self.global_scope.as_window();
        let doc = window.Document();
        match doc.GetBody() {
            Some(body_element) => {
                let element = body_element.upcast::<Element>();
                let root = DomRoot::from_ref(element);
                let id = self.elements.len() as u32;
                self.elements.push(Some(root));
                Ok(Resource::new_own(id))
            },
            None => Err("Document has no body element".to_string()),
        }
    }

    fn get_element_by_id(
        &mut self,
        id: String,
    ) -> Result<Resource<wit_dom::servo::dom::document::Element>, String> {
        let window = self.global_scope.as_window();
        let doc = window.Document();
        let dom_id = DOMString::from(id);
        let cx = unsafe { &mut *self.cx };

        match doc.GetElementById(cx, dom_id) {
            Some(element) => {
                let id = self.elements.len() as u32;
                self.elements.push(Some(element));
                Ok(Resource::new_own(id))
            },
            None => Err("Element not found".to_string()),
        }
    }

    fn query_selector(
        &mut self,
        selector: String,
    ) -> Option<Resource<wit_dom::servo::dom::document::Element>> {
        let window = self.global_scope.as_window();
        let doc = window.Document();
        let cx = unsafe { &mut *self.cx };

        if let Ok(Some(elem)) = doc.QuerySelector(cx, DOMString::from(selector)) {
            let id = self.elements.len() as u32;
            self.elements.push(Some(elem));
            Some(Resource::new_own(id))
        } else {
            None
        }
    }
}

// 5. Runner entrypoint invoked from HTMLScriptElement::execute
pub fn run_wasm_component(
    cx: &mut JSContext,
    wasm_bytes: &[u8],
    global_scope: DomRoot<GlobalScope>,
) -> wasmtime::Result<()> {
    println!(
        "[Servo Wasm Debug]: run_wasm_component received {} bytes",
        wasm_bytes.len()
    );
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let component = Component::from_binary(&engine, wasm_bytes)?;

    let mut linker = Linker::new(&engine);
    App::add_to_linker::<_, HasSelf<_>>(&mut linker, |state: &mut WasmHostState| state)?;

    let mut store = Store::new(&engine, WasmHostState::new(cx, global_scope));

    let app = App::instantiate(&mut store, &component, &linker)?;

    println!("[Servo Wasm Host]: Starting component execution...");
    app.call_run(&mut store)?;
    println!("[Servo Wasm Host]: Component execution finished successfully.");

    // Preserve the Store and App instance so event clicks can invoke on_event
    ACTIVE_WASM_APPS.with(|apps| {
        apps.borrow_mut().push(Some((store, app)));
    });

    Ok(())
}
