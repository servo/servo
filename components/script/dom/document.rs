/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use document_loader::{DocumentLoader, LoadType};
use dom::activation::{ActivationSource, synthetic_click_activation};
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use dom::bindings::codegen::Bindings::DocumentBinding;
use dom::bindings::codegen::Bindings::DocumentBinding::{DocumentMethods, DocumentReadyState};
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventHandlerBinding::OnErrorEventHandlerNonNull;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceMethods;
use dom::bindings::codegen::Bindings::TouchBinding::TouchMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap, Root};
use dom::bindings::js::RootedReference;
use dom::bindings::num::Finite;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::bindings::xmlname::{namespace_from_domstring, validate_and_extract, xml_name_type};
use dom::bindings::xmlname::XMLName::InvalidXMLName;
use dom::browsingcontext::BrowsingContext;
use dom::closeevent::CloseEvent;
use dom::comment::Comment;
use dom::customevent::CustomEvent;
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::domimplementation::DOMImplementation;
use dom::element::{Element, ElementCreator};
use dom::errorevent::ErrorEvent;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventdispatcher::EventStatus;
use dom::eventtarget::EventTarget;
use dom::focusevent::FocusEvent;
use dom::forcetouchevent::ForceTouchEvent;
use dom::globalscope::GlobalScope;
use dom::hashchangeevent::HashChangeEvent;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlappletelement::HTMLAppletElement;
use dom::htmlareaelement::HTMLAreaElement;
use dom::htmlbaseelement::HTMLBaseElement;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::HTMLElement;
use dom::htmlembedelement::HTMLEmbedElement;
use dom::htmlformelement::HTMLFormElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmllinkelement::HTMLLinkElement;
use dom::htmlmetaelement::HTMLMetaElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmlstyleelement::HTMLStyleElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::keyboardevent::KeyboardEvent;
use dom::location::Location;
use dom::messageevent::MessageEvent;
use dom::mouseevent::MouseEvent;
use dom::node::{self, CloneChildrenFlag, Node, NodeDamage, window_from_node};
use dom::nodeiterator::NodeIterator;
use dom::nodelist::NodeList;
use dom::pagetransitionevent::PageTransitionEvent;
use dom::popstateevent::PopStateEvent;
use dom::processinginstruction::ProcessingInstruction;
use dom::progressevent::ProgressEvent;
use dom::range::Range;
use dom::servoparser::ServoParser;
use dom::storageevent::StorageEvent;
use dom::stylesheetlist::StyleSheetList;
use dom::text::Text;
use dom::touch::Touch;
use dom::touchevent::TouchEvent;
use dom::touchlist::TouchList;
use dom::treewalker::TreeWalker;
use dom::uievent::UIEvent;
use dom::webglcontextevent::WebGLContextEvent;
use dom::window::{ReflowReason, Window};
use encoding::EncodingRef;
use encoding::all::UTF_8;
use euclid::point::Point2D;
use html5ever::tree_builder::{LimitedQuirks, NoQuirks, Quirks, QuirksMode};
use html5ever_atoms::{LocalName, QualName};
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{JSContext, JSObject, JSRuntime};
use js::jsapi::JS_GetRuntime;
use msg::constellation_msg::{ALT, CONTROL, SHIFT, SUPER};
use msg::constellation_msg::{FrameId, Key, KeyModifiers, KeyState};
use net_traits::{FetchResponseMsg, IpcSend, ReferrerPolicy};
use net_traits::CookieSource::NonHTTP;
use net_traits::CoreResourceMsg::{GetCookiesForUrl, SetCookiesForUrl};
use net_traits::request::RequestInit;
use net_traits::response::HttpsState;
use num_traits::ToPrimitive;
use origin::Origin;
use script_layout_interface::message::{Msg, ReflowQueryType};
use script_thread::{MainThreadScriptMsg, Runnable};
use script_traits::{AnimationState, CompositorEvent, MouseButton, MouseEventType, MozBrowserEvent};
use script_traits::{ScriptMsg as ConstellationMsg, TouchpadPressurePhase};
use script_traits::{TouchEventType, TouchId};
use script_traits::UntrustedNodeAddress;
use servo_atoms::Atom;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::boxed::FnBox;
use std::cell::{Cell, Ref, RefMut};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::default::Default;
use std::iter::once;
use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use style::attr::AttrValue;
use style::context::ReflowGoal;
use style::selector_impl::ElementSnapshot;
use style::str::{split_html_space_chars, str_join};
use style::stylesheets::Stylesheet;
use time;
use url::Url;
use url::percent_encoding::percent_decode;
use util::prefs::PREFS;

pub enum TouchEventResult {
    Processed(bool),
    Forwarded,
}

#[derive(JSTraceable, PartialEq, HeapSizeOf)]
pub enum IsHTMLDocument {
    HTMLDocument,
    NonHTMLDocument,
}

#[derive(PartialEq)]
enum ParserBlockedByScript {
    Blocked,
    Unblocked,
}

#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
struct StylesheetInDocument {
    node: JS<Node>,
    #[ignore_heap_size_of = "Arc"]
    stylesheet: Arc<Stylesheet>,
}

// https://dom.spec.whatwg.org/#document
#[dom_struct]
pub struct Document {
    node: Node,
    window: JS<Window>,
    /// https://html.spec.whatwg.org/multipage/#concept-document-bc
    browsing_context: Option<JS<BrowsingContext>>,
    implementation: MutNullableHeap<JS<DOMImplementation>>,
    location: MutNullableHeap<JS<Location>>,
    content_type: DOMString,
    last_modified: Option<String>,
    encoding: Cell<EncodingRef>,
    is_html_document: bool,
    url: Url,
    quirks_mode: Cell<QuirksMode>,
    /// Caches for the getElement methods
    id_map: DOMRefCell<HashMap<Atom, Vec<JS<Element>>>>,
    tag_map: DOMRefCell<HashMap<LocalName, JS<HTMLCollection>>>,
    tagns_map: DOMRefCell<HashMap<QualName, JS<HTMLCollection>>>,
    classes_map: DOMRefCell<HashMap<Vec<Atom>, JS<HTMLCollection>>>,
    images: MutNullableHeap<JS<HTMLCollection>>,
    embeds: MutNullableHeap<JS<HTMLCollection>>,
    links: MutNullableHeap<JS<HTMLCollection>>,
    forms: MutNullableHeap<JS<HTMLCollection>>,
    scripts: MutNullableHeap<JS<HTMLCollection>>,
    anchors: MutNullableHeap<JS<HTMLCollection>>,
    applets: MutNullableHeap<JS<HTMLCollection>>,
    /// List of stylesheets associated with nodes in this document. |None| if the list needs to be refreshed.
    stylesheets: DOMRefCell<Option<Vec<StylesheetInDocument>>>,
    /// Whether the list of stylesheets has changed since the last reflow was triggered.
    stylesheets_changed_since_reflow: Cell<bool>,
    ready_state: Cell<DocumentReadyState>,
    /// Whether the DOMContentLoaded event has already been dispatched.
    domcontentloaded_dispatched: Cell<bool>,
    /// The element that has most recently requested focus for itself.
    possibly_focused: MutNullableHeap<JS<Element>>,
    /// The element that currently has the document focus context.
    focused: MutNullableHeap<JS<Element>>,
    /// The script element that is currently executing.
    current_script: MutNullableHeap<JS<HTMLScriptElement>>,
    /// https://html.spec.whatwg.org/multipage/#pending-parsing-blocking-script
    pending_parsing_blocking_script: MutNullableHeap<JS<HTMLScriptElement>>,
    /// Number of stylesheets that block executing the next parser-inserted script
    script_blocking_stylesheets_count: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing
    deferred_scripts: DOMRefCell<Vec<JS<HTMLScriptElement>>>,
    /// https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-in-order-as-soon-as-possible
    asap_in_order_scripts_list: DOMRefCell<Vec<JS<HTMLScriptElement>>>,
    /// https://html.spec.whatwg.org/multipage/#set-of-scripts-that-will-execute-as-soon-as-possible
    asap_scripts_set: DOMRefCell<Vec<JS<HTMLScriptElement>>>,
    /// https://html.spec.whatwg.org/multipage/#concept-n-noscript
    /// True if scripting is enabled for all scripts in this document
    scripting_enabled: Cell<bool>,
    /// https://html.spec.whatwg.org/multipage/#animation-frame-callback-identifier
    /// Current identifier of animation frame callback
    animation_frame_ident: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#list-of-animation-frame-callbacks
    /// List of animation frame callbacks
    #[ignore_heap_size_of = "closures are hard"]
    animation_frame_list: DOMRefCell<Vec<(u32, Option<Box<FnBox(f64)>>)>>,
    /// Whether we're in the process of running animation callbacks.
    ///
    /// Tracking this is not necessary for correctness. Instead, it is an optimization to avoid
    /// sending needless `ChangeRunningAnimationsState` messages to the compositor.
    running_animation_callbacks: Cell<bool>,
    /// Tracks all outstanding loads related to this document.
    loader: DOMRefCell<DocumentLoader>,
    /// The current active HTML parser, to allow resuming after interruptions.
    current_parser: MutNullableHeap<JS<ServoParser>>,
    /// When we should kick off a reflow. This happens during parsing.
    reflow_timeout: Cell<Option<u64>>,
    /// The cached first `base` element with an `href` attribute.
    base_element: MutNullableHeap<JS<HTMLBaseElement>>,
    /// This field is set to the document itself for inert documents.
    /// https://html.spec.whatwg.org/multipage/#appropriate-template-contents-owner-document
    appropriate_template_contents_owner_document: MutNullableHeap<JS<Document>>,
    /// For each element that has had a state or attribute change since the last restyle,
    /// track the original condition of the element.
    modified_elements: DOMRefCell<HashMap<JS<Element>, ElementSnapshot>>,
    /// This flag will be true if layout suppressed a reflow attempt that was
    /// needed in order for the page to be painted.
    needs_paint: Cell<bool>,
    /// http://w3c.github.io/touch-events/#dfn-active-touch-point
    active_touch_points: DOMRefCell<Vec<JS<Touch>>>,
    /// Navigation Timing properties:
    /// https://w3c.github.io/navigation-timing/#sec-PerformanceNavigationTiming
    dom_loading: Cell<u64>,
    dom_interactive: Cell<u64>,
    dom_content_loaded_event_start: Cell<u64>,
    dom_content_loaded_event_end: Cell<u64>,
    dom_complete: Cell<u64>,
    load_event_start: Cell<u64>,
    load_event_end: Cell<u64>,
    /// https://html.spec.whatwg.org/multipage/#concept-document-https-state
    https_state: Cell<HttpsState>,
    touchpad_pressure_phase: Cell<TouchpadPressurePhase>,
    /// The document's origin.
    origin: Origin,
    ///  https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-states
    referrer_policy: Cell<Option<ReferrerPolicy>>,
    /// https://html.spec.whatwg.org/multipage/#dom-document-referrer
    referrer: Option<String>,
    /// https://html.spec.whatwg.org/multipage/#target-element
    target_element: MutNullableHeap<JS<Element>>,
    /// https://w3c.github.io/uievents/#event-type-dblclick
    #[ignore_heap_size_of = "Defined in std"]
    last_click_info: DOMRefCell<Option<(Instant, Point2D<f32>)>>,
}

#[derive(JSTraceable, HeapSizeOf)]
struct ImagesFilter;
impl CollectionFilter for ImagesFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLImageElement>()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct EmbedsFilter;
impl CollectionFilter for EmbedsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLEmbedElement>()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct LinksFilter;
impl CollectionFilter for LinksFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        (elem.is::<HTMLAnchorElement>() || elem.is::<HTMLAreaElement>()) &&
        elem.has_attribute(&local_name!("href"))
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct FormsFilter;
impl CollectionFilter for FormsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLFormElement>()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct ScriptsFilter;
impl CollectionFilter for ScriptsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLScriptElement>()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct AnchorsFilter;
impl CollectionFilter for AnchorsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLAnchorElement>() && elem.has_attribute(&local_name!("href"))
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct AppletsFilter;
impl CollectionFilter for AppletsFilter {
    fn filter(&self, elem: &Element, _root: &Node) -> bool {
        elem.is::<HTMLAppletElement>()
    }
}

impl Document {
    #[inline]
    pub fn loader(&self) -> Ref<DocumentLoader> {
        self.loader.borrow()
    }

    #[inline]
    pub fn mut_loader(&self) -> RefMut<DocumentLoader> {
        self.loader.borrow_mut()
    }

    /// https://html.spec.whatwg.org/multipage/#concept-document-bc
    #[inline]
    pub fn browsing_context(&self) -> Option<&BrowsingContext> {
        self.browsing_context.as_ref().map(|browsing_context| &**browsing_context)
    }

    #[inline]
    pub fn window(&self) -> &Window {
        &*self.window
    }

    #[inline]
    pub fn is_html_document(&self) -> bool {
        self.is_html_document
    }

    pub fn set_https_state(&self, https_state: HttpsState) {
        self.https_state.set(https_state);
        self.trigger_mozbrowser_event(MozBrowserEvent::SecurityChange(https_state));
    }

    // https://html.spec.whatwg.org/multipage/#fully-active
    pub fn is_fully_active(&self) -> bool {
        let browsing_context = match self.browsing_context() {
            Some(browsing_context) => browsing_context,
            None => return false,
        };
        let active_document = browsing_context.active_document();

        if self != &*active_document {
            return false;
        }
        // FIXME: It should also check whether the browser context is top-level or not
        true
    }

    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    // https://dom.spec.whatwg.org/#concept-document-url
    pub fn url(&self) -> &Url {
        &self.url
    }

    // https://html.spec.whatwg.org/multipage/#fallback-base-url
    pub fn fallback_base_url(&self) -> Url {
        // Step 1: iframe srcdoc (#4767).
        // Step 2: about:blank with a creator browsing context.
        // Step 3.
        self.url().clone()
    }

    // https://html.spec.whatwg.org/multipage/#document-base-url
    pub fn base_url(&self) -> Url {
        match self.base_element() {
            // Step 1.
            None => self.fallback_base_url(),
            // Step 2.
            Some(base) => base.frozen_base_url(),
        }
    }

    pub fn needs_paint(&self) -> bool {
        self.needs_paint.get()
    }

    pub fn needs_reflow(&self) -> bool {
        // FIXME: This should check the dirty bit on the document,
        // not the document element. Needs some layout changes to make
        // that workable.
        match self.GetDocumentElement() {
            Some(root) => {
                root.upcast::<Node>().is_dirty() ||
                root.upcast::<Node>().has_dirty_descendants() ||
                !self.modified_elements.borrow().is_empty() ||
                self.needs_paint()
            }
            None => false,
        }
    }

    /// Returns the first `base` element in the DOM that has an `href` attribute.
    pub fn base_element(&self) -> Option<Root<HTMLBaseElement>> {
        self.base_element.get()
    }

    /// Refresh the cached first base element in the DOM.
    /// https://github.com/w3c/web-platform-tests/issues/2122
    pub fn refresh_base_element(&self) {
        let base = self.upcast::<Node>()
                       .traverse_preorder()
                       .filter_map(Root::downcast::<HTMLBaseElement>)
                       .find(|element| element.upcast::<Element>().has_attribute(&local_name!("href")));
        self.base_element.set(base.r());
    }

    pub fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode.get()
    }

    pub fn set_quirks_mode(&self, mode: QuirksMode) {
        self.quirks_mode.set(mode);

        if mode == Quirks {
            self.window.layout_chan().send(Msg::SetQuirksMode).unwrap();
        }
    }

    pub fn encoding(&self) -> EncodingRef {
        self.encoding.get()
    }

    pub fn set_encoding(&self, encoding: EncodingRef) {
        self.encoding.set(encoding);
    }

    pub fn content_and_heritage_changed(&self, node: &Node, damage: NodeDamage) {
        node.force_dirty_ancestors(damage);
    }

    /// Reflows and disarms the timer if the reflow timer has expired.
    pub fn reflow_if_reflow_timer_expired(&self) {
        if let Some(reflow_timeout) = self.reflow_timeout.get() {
            if time::precise_time_ns() < reflow_timeout {
                return;
            }

            self.reflow_timeout.set(None);
            self.window.reflow(ReflowGoal::ForDisplay,
                               ReflowQueryType::NoQuery,
                               ReflowReason::RefreshTick);
        }
    }

    /// Schedules a reflow to be kicked off at the given `timeout` (in `time::precise_time_ns()`
    /// units). This reflow happens even if the event loop is busy. This is used to display initial
    /// page content during parsing.
    pub fn set_reflow_timeout(&self, timeout: u64) {
        if let Some(existing_timeout) = self.reflow_timeout.get() {
            if existing_timeout < timeout {
                return;
            }
        }
        self.reflow_timeout.set(Some(timeout))
    }

    /// Disables any pending reflow timeouts.
    pub fn disarm_reflow_timeout(&self) {
        self.reflow_timeout.set(None)
    }

    /// Remove any existing association between the provided id and any elements in this document.
    pub fn unregister_named_element(&self, to_unregister: &Element, id: Atom) {
        debug!("Removing named element from document {:p}: {:p} id={}",
               self,
               to_unregister,
               id);
        let mut id_map = self.id_map.borrow_mut();
        let is_empty = match id_map.get_mut(&id) {
            None => false,
            Some(elements) => {
                let position = elements.iter()
                                       .position(|element| &**element == to_unregister)
                                       .expect("This element should be in registered.");
                elements.remove(position);
                elements.is_empty()
            }
        };
        if is_empty {
            id_map.remove(&id);
        }
    }

    /// Associate an element present in this document with the provided id.
    pub fn register_named_element(&self, element: &Element, id: Atom) {
        debug!("Adding named element to document {:p}: {:p} id={}",
               self,
               element,
               id);
        assert!(element.upcast::<Node>().is_in_doc());
        assert!(!id.is_empty());

        let mut id_map = self.id_map.borrow_mut();

        let root = self.GetDocumentElement()
                       .expect("The element is in the document, so there must be a document \
                                element.");

        match id_map.entry(id) {
            Vacant(entry) => {
                entry.insert(vec![JS::from_ref(element)]);
            }
            Occupied(entry) => {
                let elements = entry.into_mut();

                let new_node = element.upcast::<Node>();
                let mut head: usize = 0;
                let root = root.upcast::<Node>();
                for node in root.traverse_preorder() {
                    if let Some(elem) = node.downcast() {
                        if &*(*elements)[head] == elem {
                            head += 1;
                        }
                        if new_node == &*node || head == elements.len() {
                            break;
                        }
                    }
                }

                elements.insert(head, JS::from_ref(element));
            }
        }
    }

    /// Attempt to find a named element in this page's document.
    /// https://html.spec.whatwg.org/multipage/#the-indicated-part-of-the-document
    pub fn find_fragment_node(&self, fragid: &str) -> Option<Root<Element>> {
        // Step 1 is not handled here; the fragid is already obtained by the calling function
        // Step 2
        if fragid.is_empty() {
            self.GetDocumentElement()
        } else {
            // Step 3 & 4
            percent_decode(fragid.as_bytes()).decode_utf8().ok()
                // Step 5
                .and_then(|decoded_fragid| self.get_element_by_id(&Atom::from(decoded_fragid)))
                // Step 6
                .or_else(|| self.get_anchor_by_name(fragid))
                // Step 7
                .or_else(|| if fragid.to_lowercase() == "top" {
                    self.GetDocumentElement()
                } else {
                    // Step 8
                    None
                })
        }
    }

    fn get_anchor_by_name(&self, name: &str) -> Option<Root<Element>> {
        let check_anchor = |node: &HTMLAnchorElement| {
            let elem = node.upcast::<Element>();
            elem.get_attribute(&ns!(), &local_name!("name"))
                .map_or(false, |attr| &**attr.value() == name)
        };
        let doc_node = self.upcast::<Node>();
        doc_node.traverse_preorder()
                .filter_map(Root::downcast)
                .find(|node| check_anchor(&node))
                .map(Root::upcast)
    }

    // https://html.spec.whatwg.org/multipage/#current-document-readiness
    pub fn set_ready_state(&self, state: DocumentReadyState) {
        match state {
            DocumentReadyState::Loading => {
                // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserconnected
                self.trigger_mozbrowser_event(MozBrowserEvent::Connected);
                update_with_current_time_ms(&self.dom_loading);
            },
            DocumentReadyState::Complete => {
                // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadend
                self.trigger_mozbrowser_event(MozBrowserEvent::LoadEnd);
                update_with_current_time_ms(&self.dom_complete);
            },
            DocumentReadyState::Interactive => update_with_current_time_ms(&self.dom_interactive),
        };

        self.ready_state.set(state);

        self.upcast::<EventTarget>().fire_event(atom!("readystatechange"));
    }

    /// Return whether scripting is enabled or not
    pub fn is_scripting_enabled(&self) -> bool {
        self.scripting_enabled.get()
    }

    /// Return the element that currently has focus.
    // https://w3c.github.io/uievents/#events-focusevent-doc-focus
    pub fn get_focused_element(&self) -> Option<Root<Element>> {
        self.focused.get()
    }

    /// Initiate a new round of checking for elements requesting focus. The last element to call
    /// `request_focus` before `commit_focus_transaction` is called will receive focus.
    pub fn begin_focus_transaction(&self) {
        self.possibly_focused.set(None);
    }

    /// Request that the given element receive focus once the current transaction is complete.
    pub fn request_focus(&self, elem: &Element) {
        if elem.is_focusable_area() {
            self.possibly_focused.set(Some(elem))
        }
    }

    /// Reassign the focus context to the element that last requested focus during this
    /// transaction, or none if no elements requested it.
    pub fn commit_focus_transaction(&self, focus_type: FocusType) {
        if self.focused == self.possibly_focused.get().r() {
            return
        }
        if let Some(ref elem) = self.focused.get() {
            let node = elem.upcast::<Node>();
            elem.set_focus_state(false);
            // FIXME: pass appropriate relatedTarget
            self.fire_focus_event(FocusEventType::Blur, node, None);
        }

        self.focused.set(self.possibly_focused.get().r());

        if let Some(ref elem) = self.focused.get() {
            elem.set_focus_state(true);
            let node = elem.upcast::<Node>();
            // FIXME: pass appropriate relatedTarget
            self.fire_focus_event(FocusEventType::Focus, node, None);
            // Update the focus state for all elements in the focus chain.
            // https://html.spec.whatwg.org/multipage/#focus-chain
            if focus_type == FocusType::Element {
                let global_scope = self.window.upcast::<GlobalScope>();
                let event = ConstellationMsg::Focus(global_scope.pipeline_id());
                global_scope.constellation_chan().send(event).unwrap();
            }
        }
    }

    /// Handles any updates when the document's title has changed.
    pub fn title_changed(&self) {
        if self.browsing_context().is_some() {
            // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsertitlechange
            self.trigger_mozbrowser_event(MozBrowserEvent::TitleChange(String::from(self.Title())));

            self.send_title_to_compositor();
        }
    }

    /// Sends this document's title to the compositor.
    pub fn send_title_to_compositor(&self) {
        let window = self.window();
        let global_scope = window.upcast::<GlobalScope>();
        global_scope
              .constellation_chan()
              .send(ConstellationMsg::SetTitle(global_scope.pipeline_id(),
                                               Some(String::from(self.Title()))))
              .unwrap();
    }

    pub fn dirty_all_nodes(&self) {
        let root = self.upcast::<Node>();
        for node in root.traverse_preorder() {
            node.dirty(NodeDamage::OtherNodeDamage)
        }
    }

    pub fn handle_mouse_event(&self,
                              js_runtime: *mut JSRuntime,
                              button: MouseButton,
                              client_point: Point2D<f32>,
                              mouse_event_type: MouseEventType) {
        let mouse_event_type_string = match mouse_event_type {
            MouseEventType::Click => "click".to_owned(),
            MouseEventType::MouseUp => "mouseup".to_owned(),
            MouseEventType::MouseDown => "mousedown".to_owned(),
        };
        debug!("{}: at {:?}", mouse_event_type_string, client_point);

        let node = match self.window.hit_test_query(client_point, false) {
            Some(node_address) => {
                debug!("node address is {:?}", node_address);
                node::from_untrusted_node_address(js_runtime, node_address)
            },
            None => return,
        };

        let el = match node.downcast::<Element>() {
            Some(el) => Root::from_ref(el),
            None => {
                let parent = node.GetParentNode();
                match parent.and_then(Root::downcast::<Element>) {
                    Some(parent) => parent,
                    None => return,
                }
            },
        };

        // If the target is an iframe, forward the event to the child document.
        if let Some(iframe) = el.downcast::<HTMLIFrameElement>() {
            if let Some(pipeline_id) = iframe.pipeline_id() {
                let rect = iframe.upcast::<Element>().GetBoundingClientRect();
                let child_origin = Point2D::new(rect.X() as f32, rect.Y() as f32);
                let child_point = client_point - child_origin;

                let event = CompositorEvent::MouseButtonEvent(mouse_event_type, button, child_point);
                let event = ConstellationMsg::ForwardEvent(pipeline_id, event);
                self.window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();
            }
            return;
        }

        let node = el.upcast::<Node>();
        debug!("{} on {:?}", mouse_event_type_string, node.debug_str());
        // Prevent click event if form control element is disabled.
        if let MouseEventType::Click = mouse_event_type {
            if el.click_event_filter_by_disabled_state() {
                return;
            }

            self.begin_focus_transaction();
        }

        // https://w3c.github.io/uievents/#event-type-click
        let client_x = client_point.x as i32;
        let client_y = client_point.y as i32;
        let click_count = 1;
        let event = MouseEvent::new(&self.window,
                                    DOMString::from(mouse_event_type_string),
                                    EventBubbles::Bubbles,
                                    EventCancelable::Cancelable,
                                    Some(&self.window),
                                    click_count,
                                    client_x,
                                    client_y,
                                    client_x,
                                    client_y, // TODO: Get real screen coordinates?
                                    false,
                                    false,
                                    false,
                                    false,
                                    0i16,
                                    None);
        let event = event.upcast::<Event>();

        // https://w3c.github.io/uievents/#trusted-events
        event.set_trusted(true);
        // https://html.spec.whatwg.org/multipage/#run-authentic-click-activation-steps
        let activatable = el.as_maybe_activatable();
        match mouse_event_type {
            MouseEventType::Click => el.authentic_click_activation(event),
            MouseEventType::MouseDown => {
                if let Some(a) = activatable {
                    a.enter_formal_activation_state();
                }

                let target = node.upcast();
                event.fire(target);
            },
            MouseEventType::MouseUp => {
                if let Some(a) = activatable {
                    a.exit_formal_activation_state();
                }

                let target = node.upcast();
                event.fire(target);
            },
        }

        if let MouseEventType::Click = mouse_event_type {
            self.commit_focus_transaction(FocusType::Element);
            self.maybe_fire_dblclick(client_point, node);
        }

        self.window.reflow(ReflowGoal::ForDisplay,
                           ReflowQueryType::NoQuery,
                           ReflowReason::MouseEvent);
    }

    fn maybe_fire_dblclick(&self, click_pos: Point2D<f32>, target: &Node) {
        // https://w3c.github.io/uievents/#event-type-dblclick
        let now = Instant::now();

        let opt = self.last_click_info.borrow_mut().take();

        if let Some((last_time, last_pos)) = opt {
            let DBL_CLICK_TIMEOUT = Duration::from_millis(PREFS.get("dom.document.dblclick_timeout").as_u64()
                                                                                                    .unwrap_or(300));
            let DBL_CLICK_DIST_THRESHOLD = PREFS.get("dom.document.dblclick_dist").as_u64().unwrap_or(1);

            // Calculate distance between this click and the previous click.
            let line = click_pos - last_pos;
            let dist = (line.dot(line) as f64).sqrt();

            if  now.duration_since(last_time) < DBL_CLICK_TIMEOUT &&
                dist < DBL_CLICK_DIST_THRESHOLD as f64 {
                // A double click has occurred if this click is within a certain time and dist. of previous click.
                let click_count = 2;
                let client_x = click_pos.x as i32;
                let client_y = click_pos.y as i32;

                let event = MouseEvent::new(&self.window,
                                            DOMString::from("dblclick"),
                                            EventBubbles::Bubbles,
                                            EventCancelable::Cancelable,
                                            Some(&self.window),
                                            click_count,
                                            client_x,
                                            client_y,
                                            client_x,
                                            client_y,
                                            false,
                                            false,
                                            false,
                                            false,
                                            0i16,
                                            None);
                event.upcast::<Event>().fire(target.upcast());

                // When a double click occurs, self.last_click_info is left as None so that a
                // third sequential click will not cause another double click.
                return;
            }
        }

        // Update last_click_info with the time and position of the click.
        *self.last_click_info.borrow_mut() = Some((now, click_pos));
    }

    pub fn handle_touchpad_pressure_event(&self,
                                          js_runtime: *mut JSRuntime,
                                          client_point: Point2D<f32>,
                                          pressure: f32,
                                          phase_now: TouchpadPressurePhase) {
        let node = match self.window.hit_test_query(client_point, false) {
            Some(node_address) => node::from_untrusted_node_address(js_runtime, node_address),
            None => return
        };

        let el = match node.downcast::<Element>() {
            Some(el) => Root::from_ref(el),
            None => {
                let parent = node.GetParentNode();
                match parent.and_then(Root::downcast::<Element>) {
                    Some(parent) => parent,
                    None => return
                }
            },
        };

        // If the target is an iframe, forward the event to the child document.
        if let Some(iframe) = el.downcast::<HTMLIFrameElement>() {
            if let Some(pipeline_id) = iframe.pipeline_id() {
                let rect = iframe.upcast::<Element>().GetBoundingClientRect();
                let child_origin = Point2D::new(rect.X() as f32, rect.Y() as f32);
                let child_point = client_point - child_origin;

                let event = CompositorEvent::TouchpadPressureEvent(child_point,
                                                                   pressure,
                                                                   phase_now);
                let event = ConstellationMsg::ForwardEvent(pipeline_id, event);
                self.window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();
            }
            return;
        }

        let phase_before = self.touchpad_pressure_phase.get();
        self.touchpad_pressure_phase.set(phase_now);

        if phase_before == TouchpadPressurePhase::BeforeClick &&
           phase_now == TouchpadPressurePhase::BeforeClick {
            return;
        }

        let node = el.upcast::<Node>();
        let target = node.upcast();

        let force = match phase_now {
            TouchpadPressurePhase::BeforeClick => pressure,
            TouchpadPressurePhase::AfterFirstClick => 1. + pressure,
            TouchpadPressurePhase::AfterSecondClick => 2. + pressure,
        };

        if phase_now != TouchpadPressurePhase::BeforeClick {
            self.fire_forcetouch_event("servomouseforcechanged".to_owned(), target, force);
        }

        if phase_before != TouchpadPressurePhase::AfterSecondClick &&
           phase_now == TouchpadPressurePhase::AfterSecondClick {
            self.fire_forcetouch_event("servomouseforcedown".to_owned(), target, force);
        }

        if phase_before == TouchpadPressurePhase::AfterSecondClick &&
           phase_now != TouchpadPressurePhase::AfterSecondClick {
            self.fire_forcetouch_event("servomouseforceup".to_owned(), target, force);
        }
    }

    fn fire_forcetouch_event(&self, event_name: String, target: &EventTarget, force: f32) {
        let force_event = ForceTouchEvent::new(&self.window,
                                               DOMString::from(event_name),
                                               force);
        let event = force_event.upcast::<Event>();
        event.fire(target);
    }

    pub fn fire_mouse_event(&self, client_point: Point2D<f32>, target: &EventTarget, event_name: String) {
        let client_x = client_point.x.to_i32().unwrap_or(0);
        let client_y = client_point.y.to_i32().unwrap_or(0);

        let mouse_event = MouseEvent::new(&self.window,
                                          DOMString::from(event_name),
                                          EventBubbles::Bubbles,
                                          EventCancelable::Cancelable,
                                          Some(&self.window),
                                          0i32,
                                          client_x,
                                          client_y,
                                          client_x,
                                          client_y,
                                          false,
                                          false,
                                          false,
                                          false,
                                          0i16,
                                          None);
        let event = mouse_event.upcast::<Event>();
        event.fire(target);
    }

    pub fn handle_mouse_move_event(&self,
                                   js_runtime: *mut JSRuntime,
                                   client_point: Option<Point2D<f32>>,
                                   prev_mouse_over_target: &MutNullableHeap<JS<Element>>) {
        let client_point = match client_point {
            None => {
                // If there's no point, there's no target under the mouse
                // FIXME: dispatch mouseout here. We have no point.
                prev_mouse_over_target.set(None);
                return;
            }
            Some(client_point) => client_point,
        };

        let maybe_new_target = self.window.hit_test_query(client_point, true).and_then(|address| {
            let node = node::from_untrusted_node_address(js_runtime, address);
            node.inclusive_ancestors()
                .filter_map(Root::downcast::<Element>)
                .next()
        });

        // Send mousemove event to topmost target, and forward it if it's an iframe
        if let Some(ref new_target) = maybe_new_target {
            // If the target is an iframe, forward the event to the child document.
            if let Some(iframe) = new_target.downcast::<HTMLIFrameElement>() {
                if let Some(pipeline_id) = iframe.pipeline_id() {
                    let rect = iframe.upcast::<Element>().GetBoundingClientRect();
                    let child_origin = Point2D::new(rect.X() as f32, rect.Y() as f32);
                    let child_point = client_point - child_origin;

                    let event = CompositorEvent::MouseMoveEvent(Some(child_point));
                    let event = ConstellationMsg::ForwardEvent(pipeline_id, event);
                    self.window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();
                }
                return;
            }

            self.fire_mouse_event(client_point, new_target.upcast(), "mousemove".to_owned());
        }

        // Nothing more to do here, mousemove is sent,
        // and the element under the mouse hasn't changed.
        if maybe_new_target == prev_mouse_over_target.get() {
            return;
        }

        let old_target_is_ancestor_of_new_target = match (prev_mouse_over_target.get(), maybe_new_target.as_ref()) {
            (Some(old_target), Some(new_target))
                => old_target.upcast::<Node>().is_ancestor_of(new_target.upcast::<Node>()),
            _   => false,
        };

        // Here we know the target has changed, so we must update the state,
        // dispatch mouseout to the previous one, mouseover to the new one,
        if let Some(old_target) = prev_mouse_over_target.get() {
            // If the old target is an ancestor of the new target, this can be skipped
            // completely, since the node's hover state will be reseted below.
            if !old_target_is_ancestor_of_new_target {
                for element in old_target.upcast::<Node>()
                                         .inclusive_ancestors()
                                         .filter_map(Root::downcast::<Element>) {
                    element.set_hover_state(false);
                    element.set_active_state(false);
                }
            }

            // Remove hover state to old target and its parents
            self.fire_mouse_event(client_point, old_target.upcast(), "mouseout".to_owned());

            // TODO: Fire mouseleave here only if the old target is
            // not an ancestor of the new target.
        }

        if let Some(ref new_target) = maybe_new_target {
            for element in new_target.upcast::<Node>()
                                     .inclusive_ancestors()
                                     .filter_map(Root::downcast::<Element>) {
                if element.hover_state() {
                    break;
                }

                element.set_hover_state(true);
            }

            self.fire_mouse_event(client_point, &new_target.upcast(), "mouseover".to_owned());

            // TODO: Fire mouseenter here.
        }

        // Store the current mouse over target for next frame.
        prev_mouse_over_target.set(maybe_new_target.r());

        self.window.reflow(ReflowGoal::ForDisplay,
                           ReflowQueryType::NoQuery,
                           ReflowReason::MouseEvent);
    }

    pub fn handle_touch_event(&self,
                              js_runtime: *mut JSRuntime,
                              event_type: TouchEventType,
                              touch_id: TouchId,
                              point: Point2D<f32>)
                              -> TouchEventResult {
        let TouchId(identifier) = touch_id;

        let event_name = match event_type {
            TouchEventType::Down => "touchstart",
            TouchEventType::Move => "touchmove",
            TouchEventType::Up => "touchend",
            TouchEventType::Cancel => "touchcancel",
        };

        let node = match self.window.hit_test_query(point, false) {
            Some(node_address) => node::from_untrusted_node_address(js_runtime, node_address),
            None => return TouchEventResult::Processed(false),
        };
        let el = match node.downcast::<Element>() {
            Some(el) => Root::from_ref(el),
            None => {
                let parent = node.GetParentNode();
                match parent.and_then(Root::downcast::<Element>) {
                    Some(parent) => parent,
                    None => return TouchEventResult::Processed(false),
                }
            },
        };

        // If the target is an iframe, forward the event to the child document.
        if let Some(iframe) = el.downcast::<HTMLIFrameElement>() {
            if let Some(pipeline_id) = iframe.pipeline_id() {
                let rect = iframe.upcast::<Element>().GetBoundingClientRect();
                let child_origin = Point2D::new(rect.X() as f32, rect.Y() as f32);
                let child_point = point - child_origin;

                let event = CompositorEvent::TouchEvent(event_type, touch_id, child_point);
                let event = ConstellationMsg::ForwardEvent(pipeline_id, event);
                self.window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();
            }
            return TouchEventResult::Forwarded;
        }

        let target = Root::upcast::<EventTarget>(el);
        let window = &*self.window;

        let client_x = Finite::wrap(point.x as f64);
        let client_y = Finite::wrap(point.y as f64);
        let page_x = Finite::wrap(point.x as f64 + window.PageXOffset() as f64);
        let page_y = Finite::wrap(point.y as f64 + window.PageYOffset() as f64);

        let touch = Touch::new(window,
                               identifier,
                               &target,
                               client_x,
                               client_y, // TODO: Get real screen coordinates?
                               client_x,
                               client_y,
                               page_x,
                               page_y);

        match event_type {
            TouchEventType::Down => {
                // Add a new touch point
                self.active_touch_points.borrow_mut().push(JS::from_ref(&*touch));
            }
            TouchEventType::Move => {
                // Replace an existing touch point
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                match active_touch_points.iter_mut().find(|t| t.Identifier() == identifier) {
                    Some(t) => *t = JS::from_ref(&*touch),
                    None => warn!("Got a touchmove event for a non-active touch point"),
                }
            }
            TouchEventType::Up |
            TouchEventType::Cancel => {
                // Remove an existing touch point
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                match active_touch_points.iter().position(|t| t.Identifier() == identifier) {
                    Some(i) => {
                        active_touch_points.swap_remove(i);
                    }
                    None => warn!("Got a touchend event for a non-active touch point"),
                }
            }
        }

        rooted_vec!(let mut touches);
        touches.extend(self.active_touch_points.borrow().iter().cloned());
        rooted_vec!(let mut target_touches);
        target_touches.extend(self.active_touch_points
                                  .borrow()
                                  .iter()
                                  .filter(|t| t.Target() == target)
                                  .cloned());
        rooted_vec!(let changed_touches <- once(touch));

        let event = TouchEvent::new(window,
                                    DOMString::from(event_name),
                                    EventBubbles::Bubbles,
                                    EventCancelable::Cancelable,
                                    Some(window),
                                    0i32,
                                    &TouchList::new(window, touches.r()),
                                    &TouchList::new(window, changed_touches.r()),
                                    &TouchList::new(window, target_touches.r()),
                                    // FIXME: modifier keys
                                    false,
                                    false,
                                    false,
                                    false);
        let event = event.upcast::<Event>();
        let result = event.fire(&target);

        window.reflow(ReflowGoal::ForDisplay,
                      ReflowQueryType::NoQuery,
                      ReflowReason::MouseEvent);

        match result {
            EventStatus::Canceled => TouchEventResult::Processed(false),
            EventStatus::NotCanceled => TouchEventResult::Processed(true),
        }
    }

    /// The entry point for all key processing for web content
    pub fn dispatch_key_event(&self,
                              ch: Option<char>,
                              key: Key,
                              state: KeyState,
                              modifiers: KeyModifiers,
                              constellation: &IpcSender<ConstellationMsg>) {
        let focused = self.get_focused_element();
        let body = self.GetBody();

        let target = match (&focused, &body) {
            (&Some(ref focused), _) => focused.upcast(),
            (&None, &Some(ref body)) => body.upcast(),
            (&None, &None) => self.window.upcast(),
        };

        let ctrl = modifiers.contains(CONTROL);
        let alt = modifiers.contains(ALT);
        let shift = modifiers.contains(SHIFT);
        let meta = modifiers.contains(SUPER);

        let is_composing = false;
        let is_repeating = state == KeyState::Repeated;
        let ev_type = DOMString::from(match state {
                                          KeyState::Pressed | KeyState::Repeated => "keydown",
                                          KeyState::Released => "keyup",
                                      }
                                      .to_owned());

        let props = KeyboardEvent::key_properties(ch, key, modifiers);

        let keyevent = KeyboardEvent::new(&self.window,
                                          ev_type,
                                          true,
                                          true,
                                          Some(&self.window),
                                          0,
                                          ch,
                                          Some(key),
                                          DOMString::from(props.key_string.clone()),
                                          DOMString::from(props.code),
                                          props.location,
                                          is_repeating,
                                          is_composing,
                                          ctrl,
                                          alt,
                                          shift,
                                          meta,
                                          None,
                                          props.key_code);
        let event = keyevent.upcast::<Event>();
        event.fire(target);
        let mut prevented = event.DefaultPrevented();

        // https://w3c.github.io/uievents/#keys-cancelable-keys
        if state != KeyState::Released && props.is_printable() && !prevented {
            // https://w3c.github.io/uievents/#keypress-event-order
            let event = KeyboardEvent::new(&self.window,
                                           DOMString::from("keypress"),
                                           true,
                                           true,
                                           Some(&self.window),
                                           0,
                                           ch,
                                           Some(key),
                                           DOMString::from(props.key_string),
                                           DOMString::from(props.code),
                                           props.location,
                                           is_repeating,
                                           is_composing,
                                           ctrl,
                                           alt,
                                           shift,
                                           meta,
                                           props.char_code,
                                           0);
            let ev = event.upcast::<Event>();
            ev.fire(target);
            prevented = ev.DefaultPrevented();
            // TODO: if keypress event is canceled, prevent firing input events
        }

        if !prevented {
            constellation.send(ConstellationMsg::SendKeyEvent(ch, key, state, modifiers)).unwrap();
        }

        // This behavior is unspecced
        // We are supposed to dispatch synthetic click activation for Space and/or Return,
        // however *when* we do it is up to us
        // I'm dispatching it after the key event so the script has a chance to cancel it
        // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27337
        match key {
            Key::Space if !prevented && state == KeyState::Released => {
                let maybe_elem = target.downcast::<Element>();
                if let Some(el) = maybe_elem {
                    synthetic_click_activation(el,
                                               false,
                                               false,
                                               false,
                                               false,
                                               ActivationSource::NotFromClick)
                }
            }
            Key::Enter if !prevented && state == KeyState::Released => {
                let maybe_elem = target.downcast::<Element>();
                if let Some(el) = maybe_elem {
                    if let Some(a) = el.as_maybe_activatable() {
                        a.implicit_submission(ctrl, alt, shift, meta);
                    }
                }
            }
            _ => (),
        }

        self.window.reflow(ReflowGoal::ForDisplay,
                           ReflowQueryType::NoQuery,
                           ReflowReason::KeyEvent);
    }

    // https://dom.spec.whatwg.org/#converting-nodes-into-a-node
    pub fn node_from_nodes_and_strings(&self,
                                       mut nodes: Vec<NodeOrString>)
                                       -> Fallible<Root<Node>> {
        if nodes.len() == 1 {
            Ok(match nodes.pop().unwrap() {
                NodeOrString::Node(node) => node,
                NodeOrString::String(string) => Root::upcast(self.CreateTextNode(string)),
            })
        } else {
            let fragment = Root::upcast::<Node>(self.CreateDocumentFragment());
            for node in nodes {
                match node {
                    NodeOrString::Node(node) => {
                        try!(fragment.AppendChild(&node));
                    },
                    NodeOrString::String(string) => {
                        let node = Root::upcast::<Node>(self.CreateTextNode(string));
                        // No try!() here because appending a text node
                        // should not fail.
                        fragment.AppendChild(&node).unwrap();
                    }
                }
            }
            Ok(fragment)
        }
    }

    pub fn get_body_attribute(&self, local_name: &LocalName) -> DOMString {
        match self.GetBody().and_then(Root::downcast::<HTMLBodyElement>) {
            Some(ref body) => {
                body.upcast::<Element>().get_string_attribute(local_name)
            },
            None => DOMString::new(),
        }
    }

    pub fn set_body_attribute(&self, local_name: &LocalName, value: DOMString) {
        if let Some(ref body) = self.GetBody().and_then(Root::downcast::<HTMLBodyElement>) {
            let body = body.upcast::<Element>();
            let value = body.parse_attribute(&ns!(), &local_name, value);
            body.set_attribute(local_name, value);
        }
    }

    pub fn set_current_script(&self, script: Option<&HTMLScriptElement>) {
        self.current_script.set(script);
    }

    pub fn get_script_blocking_stylesheets_count(&self) -> u32 {
        self.script_blocking_stylesheets_count.get()
    }

    pub fn increment_script_blocking_stylesheet_count(&self) {
        let count_cell = &self.script_blocking_stylesheets_count;
        count_cell.set(count_cell.get() + 1);
    }

    pub fn decrement_script_blocking_stylesheet_count(&self) {
        let count_cell = &self.script_blocking_stylesheets_count;
        assert!(count_cell.get() > 0);
        count_cell.set(count_cell.get() - 1);
    }

    pub fn invalidate_stylesheets(&self) {
        self.stylesheets_changed_since_reflow.set(true);
        *self.stylesheets.borrow_mut() = None;
        // Mark the document element dirty so a reflow will be performed.
        if let Some(element) = self.GetDocumentElement() {
            element.upcast::<Node>().dirty(NodeDamage::NodeStyleDamaged);
        }
    }

    pub fn get_and_reset_stylesheets_changed_since_reflow(&self) -> bool {
        let changed = self.stylesheets_changed_since_reflow.get();
        self.stylesheets_changed_since_reflow.set(false);
        changed
    }

    pub fn set_pending_parsing_blocking_script(&self, script: Option<&HTMLScriptElement>) {
        assert!(self.get_pending_parsing_blocking_script().is_none() || script.is_none());
        self.pending_parsing_blocking_script.set(script);
    }

    pub fn get_pending_parsing_blocking_script(&self) -> Option<Root<HTMLScriptElement>> {
        self.pending_parsing_blocking_script.get()
    }

    pub fn add_deferred_script(&self, script: &HTMLScriptElement) {
        self.deferred_scripts.borrow_mut().push(JS::from_ref(script));
    }

    pub fn add_asap_script(&self, script: &HTMLScriptElement) {
        self.asap_scripts_set.borrow_mut().push(JS::from_ref(script));
    }

    pub fn push_asap_in_order_script(&self, script: &HTMLScriptElement) {
        self.asap_in_order_scripts_list.borrow_mut().push(JS::from_ref(script));
    }

    pub fn trigger_mozbrowser_event(&self, event: MozBrowserEvent) {
        if PREFS.is_mozbrowser_enabled() {
            if let Some((parent_pipeline_id, _)) = self.window.parent_info() {
                let global_scope = self.window.upcast::<GlobalScope>();
                let event = ConstellationMsg::MozBrowserEvent(parent_pipeline_id,
                                                              global_scope.pipeline_id(),
                                                              event);
                global_scope.constellation_chan().send(event).unwrap();
            }
        }
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-requestanimationframe
    pub fn request_animation_frame(&self, callback: Box<FnBox(f64)>) -> u32 {
        let ident = self.animation_frame_ident.get() + 1;

        self.animation_frame_ident.set(ident);
        self.animation_frame_list.borrow_mut().push((ident, Some(callback)));

        // No need to send a `ChangeRunningAnimationsState` if we're running animation callbacks:
        // we're guaranteed to already be in the "animation callbacks present" state.
        //
        // This reduces CPU usage by avoiding needless thread wakeups in the common case of
        // repeated rAF.
        //
        // TODO: Should tick animation only when document is visible
        if !self.running_animation_callbacks.get() {
            let global_scope = self.window.upcast::<GlobalScope>();
            let event = ConstellationMsg::ChangeRunningAnimationsState(
                global_scope.pipeline_id(),
                AnimationState::AnimationCallbacksPresent);
            global_scope.constellation_chan().send(event).unwrap();
        }

        ident
    }

    /// https://html.spec.whatwg.org/multipage/#dom-window-cancelanimationframe
    pub fn cancel_animation_frame(&self, ident: u32) {
        let mut list = self.animation_frame_list.borrow_mut();
        if let Some(mut pair) = list.iter_mut().find(|pair| pair.0 == ident) {
            pair.1 = None;
        }
    }

    /// https://html.spec.whatwg.org/multipage/#run-the-animation-frame-callbacks
    pub fn run_the_animation_frame_callbacks(&self) {
        let mut animation_frame_list =
            mem::replace(&mut *self.animation_frame_list.borrow_mut(), vec![]);
        self.running_animation_callbacks.set(true);
        let timing = self.window.Performance().Now();

        for (_, callback) in animation_frame_list.drain(..) {
            if let Some(callback) = callback {
                callback(*timing);
            }
        }

        // Only send the animation change state message after running any callbacks.
        // This means that if the animation callback adds a new callback for
        // the next frame (which is the common case), we won't send a NoAnimationCallbacksPresent
        // message quickly followed by an AnimationCallbacksPresent message.
        if self.animation_frame_list.borrow().is_empty() {
            mem::swap(&mut *self.animation_frame_list.borrow_mut(),
                      &mut animation_frame_list);
            let global_scope = self.window.upcast::<GlobalScope>();
            let event = ConstellationMsg::ChangeRunningAnimationsState(global_scope.pipeline_id(),
                                                                       AnimationState::NoAnimationCallbacksPresent);
            global_scope.constellation_chan().send(event).unwrap();
        }

        self.running_animation_callbacks.set(false);

        self.window.reflow(ReflowGoal::ForDisplay,
                           ReflowQueryType::NoQuery,
                           ReflowReason::RequestAnimationFrame);
    }

    pub fn fetch_async(&self, load: LoadType,
                       request: RequestInit,
                       fetch_target: IpcSender<FetchResponseMsg>) {
        let mut loader = self.loader.borrow_mut();
        loader.fetch_async(load, request, fetch_target);
    }

    pub fn finish_load(&self, load: LoadType) {
        debug!("Document got finish_load: {:?}", load);
        // The parser might need the loader, so restrict the lifetime of the borrow.
        {
            let mut loader = self.loader.borrow_mut();
            loader.finish_load(&load);
        }

        if let LoadType::Script(_) = load {
            self.process_deferred_scripts();
            self.process_asap_scripts();
        }

        if self.maybe_execute_parser_blocking_script() == ParserBlockedByScript::Blocked {
            return;
        }

        // A finished resource load can potentially unblock parsing. In that case, resume the
        // parser so its loop can find out.
        if let Some(parser) = self.get_current_parser() {
            if parser.is_suspended() {
                parser.resume();
            }
        } else if self.reflow_timeout.get().is_none() {
            // If we don't have a parser, and the reflow timer has been reset, explicitly
            // trigger a reflow.
            if let LoadType::Stylesheet(_) = load {
                self.window.reflow(ReflowGoal::ForDisplay,
                                   ReflowQueryType::NoQuery,
                                   ReflowReason::StylesheetLoaded);
            }
        }

        if !self.loader.borrow().is_blocked() && !self.loader.borrow().events_inhibited() {
            // Schedule a task to fire a "load" event (if no blocking loads have arrived in the mean time)
            // NOTE: we can end up executing this code more than once, in case more blocking loads arrive.
            debug!("Document loads are complete.");
            let win = self.window();
            let msg = MainThreadScriptMsg::DocumentLoadsComplete(
                win.upcast::<GlobalScope>().pipeline_id());
            win.main_thread_script_chan().send(msg).unwrap();
        }
    }

    /// If document parsing is blocked on a script, and that script is ready to run,
    /// execute it.
    /// https://html.spec.whatwg.org/multipage/#ready-to-be-parser-executed
    fn maybe_execute_parser_blocking_script(&self) -> ParserBlockedByScript {
        let script = match self.pending_parsing_blocking_script.get() {
            None => return ParserBlockedByScript::Unblocked,
            Some(script) => script,
        };

        if self.script_blocking_stylesheets_count.get() == 0 && script.is_ready_to_be_executed() {
            self.pending_parsing_blocking_script.set(None);
            script.execute();
            return ParserBlockedByScript::Unblocked;
        }
        ParserBlockedByScript::Blocked
    }

    /// https://html.spec.whatwg.org/multipage/#the-end step 3
    pub fn process_deferred_scripts(&self) {
        if self.ready_state.get() != DocumentReadyState::Interactive {
            return;
        }
        // Part of substep 1.
        if self.script_blocking_stylesheets_count.get() > 0 {
            return;
        }
        let mut deferred_scripts = self.deferred_scripts.borrow_mut();
        while !deferred_scripts.is_empty() {
            {
                let script = &*deferred_scripts[0];
                // Part of substep 1.
                if !script.is_ready_to_be_executed() {
                    return;
                }
                // Substep 2.
                script.execute();
            }
            // Substep 3.
            deferred_scripts.remove(0);
            // Substep 4 (implicit).
        }
        // https://html.spec.whatwg.org/multipage/#the-end step 4.
        self.maybe_dispatch_dom_content_loaded();
    }

    /// https://html.spec.whatwg.org/multipage/#the-end step 5 and the latter parts of
    /// https://html.spec.whatwg.org/multipage/#prepare-a-script 20.d and 20.e.
    pub fn process_asap_scripts(&self) {
        // Execute the first in-order asap-executed script if it's ready, repeat as required.
        // Re-borrowing the list for each step because it can also be borrowed under execute.
        while self.asap_in_order_scripts_list.borrow().len() > 0 {
            let script = Root::from_ref(&*self.asap_in_order_scripts_list.borrow()[0]);
            if !script.is_ready_to_be_executed() {
                break;
            }
            script.execute();
            self.asap_in_order_scripts_list.borrow_mut().remove(0);
        }

        let mut idx = 0;
        // Re-borrowing the set for each step because it can also be borrowed under execute.
        while idx < self.asap_scripts_set.borrow().len() {
            let script = Root::from_ref(&*self.asap_scripts_set.borrow()[idx]);
            if !script.is_ready_to_be_executed() {
                idx += 1;
                continue;
            }
            script.execute();
            self.asap_scripts_set.borrow_mut().swap_remove(idx);
        }
    }

    pub fn maybe_dispatch_dom_content_loaded(&self) {
        if self.domcontentloaded_dispatched.get() {
            return;
        }
        self.domcontentloaded_dispatched.set(true);
        assert!(self.ReadyState() != DocumentReadyState::Complete,
                "Complete before DOMContentLoaded?");

        update_with_current_time_ms(&self.dom_content_loaded_event_start);

        let window = self.window();
        window.dom_manipulation_task_source().queue_event(self.upcast(), atom!("DOMContentLoaded"),
            EventBubbles::Bubbles, EventCancelable::NotCancelable, window);

        window.reflow(ReflowGoal::ForDisplay,
                      ReflowQueryType::NoQuery,
                      ReflowReason::DOMContentLoaded);
        update_with_current_time_ms(&self.dom_content_loaded_event_end);
    }

    pub fn notify_constellation_load(&self) {
        let global_scope = self.window.upcast::<GlobalScope>();
        let pipeline_id = global_scope.pipeline_id();
        let load_event = ConstellationMsg::LoadComplete(pipeline_id);
        global_scope.constellation_chan().send(load_event).unwrap();
    }

    pub fn set_current_parser(&self, script: Option<&ServoParser>) {
        self.current_parser.set(script);
    }

    pub fn get_current_parser(&self) -> Option<Root<ServoParser>> {
        self.current_parser.get()
    }

    /// Find an iframe element in the document.
    pub fn find_iframe(&self, frame_id: FrameId) -> Option<Root<HTMLIFrameElement>> {
        self.upcast::<Node>()
            .traverse_preorder()
            .filter_map(Root::downcast::<HTMLIFrameElement>)
            .find(|node| node.frame_id() == frame_id)
    }

    pub fn get_dom_loading(&self) -> u64 {
        self.dom_loading.get()
    }

    pub fn get_dom_interactive(&self) -> u64 {
        self.dom_interactive.get()
    }

    pub fn get_dom_content_loaded_event_start(&self) -> u64 {
        self.dom_content_loaded_event_start.get()
    }

    pub fn get_dom_content_loaded_event_end(&self) -> u64 {
        self.dom_content_loaded_event_end.get()
    }

    pub fn get_dom_complete(&self) -> u64 {
        self.dom_complete.get()
    }

    pub fn get_load_event_start(&self) -> u64 {
        self.load_event_start.get()
    }

    pub fn get_load_event_end(&self) -> u64 {
        self.load_event_end.get()
    }

    // https://html.spec.whatwg.org/multipage/#fire-a-focus-event
    fn fire_focus_event(&self, focus_event_type: FocusEventType, node: &Node, related_target: Option<&EventTarget>) {
        let (event_name, does_bubble) = match focus_event_type {
            FocusEventType::Focus => (DOMString::from("focus"), EventBubbles::DoesNotBubble),
            FocusEventType::Blur => (DOMString::from("blur"), EventBubbles::DoesNotBubble),
        };
        let event = FocusEvent::new(&self.window,
                                    event_name,
                                    does_bubble,
                                    EventCancelable::NotCancelable,
                                    Some(&self.window),
                                    0i32,
                                    related_target);
        let event = event.upcast::<Event>();
        event.set_trusted(true);
        let target = node.upcast();
        event.fire(target);
    }

    /// https://html.spec.whatwg.org/multipage/#cookie-averse-document-object
    pub fn is_cookie_averse(&self) -> bool {
        self.browsing_context.is_none() || !url_has_network_scheme(&self.url)
    }

    pub fn nodes_from_point(&self, client_point: &Point2D<f32>) -> Vec<UntrustedNodeAddress> {
        let page_point =
            Point2D::new(client_point.x + self.window.PageXOffset() as f32,
                         client_point.y + self.window.PageYOffset() as f32);

        self.window.layout().nodes_from_point(page_point, *client_point)
    }
}

#[derive(PartialEq, HeapSizeOf)]
pub enum DocumentSource {
    FromParser,
    NotFromParser,
}

#[allow(unsafe_code)]
pub trait LayoutDocumentHelpers {
    unsafe fn is_html_document_for_layout(&self) -> bool;
    unsafe fn drain_modified_elements(&self) -> Vec<(LayoutJS<Element>, ElementSnapshot)>;
    unsafe fn needs_paint_from_layout(&self);
    unsafe fn will_paint(&self);
}

#[allow(unsafe_code)]
impl LayoutDocumentHelpers for LayoutJS<Document> {
    #[inline]
    unsafe fn is_html_document_for_layout(&self) -> bool {
        (*self.unsafe_get()).is_html_document
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn drain_modified_elements(&self) -> Vec<(LayoutJS<Element>, ElementSnapshot)> {
        let mut elements = (*self.unsafe_get()).modified_elements.borrow_mut_for_layout();
        let result = elements.drain().map(|(k, v)| (k.to_layout(), v)).collect();
        result
    }

    #[inline]
    unsafe fn needs_paint_from_layout(&self) {
        (*self.unsafe_get()).needs_paint.set(true)
    }

    #[inline]
    unsafe fn will_paint(&self) {
        (*self.unsafe_get()).needs_paint.set(false)
    }
}

/// https://url.spec.whatwg.org/#network-scheme
fn url_has_network_scheme(url: &Url) -> bool {
    match url.scheme() {
        "ftp" | "http" | "https" => true,
        _ => false,
    }
}

impl Document {
    pub fn new_inherited(window: &Window,
                         browsing_context: Option<&BrowsingContext>,
                         url: Option<Url>,
                         is_html_document: IsHTMLDocument,
                         content_type: Option<DOMString>,
                         last_modified: Option<String>,
                         source: DocumentSource,
                         doc_loader: DocumentLoader,
                         referrer: Option<String>,
                         referrer_policy: Option<ReferrerPolicy>)
                         -> Document {
        let url = url.unwrap_or_else(|| Url::parse("about:blank").unwrap());

        let (ready_state, domcontentloaded_dispatched) = if source == DocumentSource::FromParser {
            (DocumentReadyState::Loading, false)
        } else {
            (DocumentReadyState::Complete, true)
        };

        // Incomplete implementation of Document origin specification at
        // https://html.spec.whatwg.org/multipage/#origin:document
        let origin = if url_has_network_scheme(&url) {
            Origin::new(&url)
        } else {
            // Default to DOM standard behaviour
            Origin::opaque_identifier()
        };

        Document {
            node: Node::new_document_node(),
            window: JS::from_ref(window),
            browsing_context: browsing_context.map(JS::from_ref),
            implementation: Default::default(),
            location: Default::default(),
            content_type: match content_type {
                Some(string) => string,
                None => DOMString::from(match is_html_document {
                    // https://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
                    IsHTMLDocument::HTMLDocument => "text/html",
                    // https://dom.spec.whatwg.org/#concept-document-content-type
                    IsHTMLDocument::NonHTMLDocument => "application/xml",
                }),
            },
            last_modified: last_modified,
            url: url,
            // https://dom.spec.whatwg.org/#concept-document-quirks
            quirks_mode: Cell::new(NoQuirks),
            // https://dom.spec.whatwg.org/#concept-document-encoding
            encoding: Cell::new(UTF_8),
            is_html_document: is_html_document == IsHTMLDocument::HTMLDocument,
            id_map: DOMRefCell::new(HashMap::new()),
            tag_map: DOMRefCell::new(HashMap::new()),
            tagns_map: DOMRefCell::new(HashMap::new()),
            classes_map: DOMRefCell::new(HashMap::new()),
            images: Default::default(),
            embeds: Default::default(),
            links: Default::default(),
            forms: Default::default(),
            scripts: Default::default(),
            anchors: Default::default(),
            applets: Default::default(),
            stylesheets: DOMRefCell::new(None),
            stylesheets_changed_since_reflow: Cell::new(false),
            ready_state: Cell::new(ready_state),
            domcontentloaded_dispatched: Cell::new(domcontentloaded_dispatched),
            possibly_focused: Default::default(),
            focused: Default::default(),
            current_script: Default::default(),
            pending_parsing_blocking_script: Default::default(),
            script_blocking_stylesheets_count: Cell::new(0u32),
            deferred_scripts: DOMRefCell::new(vec![]),
            asap_in_order_scripts_list: DOMRefCell::new(vec![]),
            asap_scripts_set: DOMRefCell::new(vec![]),
            scripting_enabled: Cell::new(browsing_context.is_some()),
            animation_frame_ident: Cell::new(0),
            animation_frame_list: DOMRefCell::new(vec![]),
            running_animation_callbacks: Cell::new(false),
            loader: DOMRefCell::new(doc_loader),
            current_parser: Default::default(),
            reflow_timeout: Cell::new(None),
            base_element: Default::default(),
            appropriate_template_contents_owner_document: Default::default(),
            modified_elements: DOMRefCell::new(HashMap::new()),
            needs_paint: Cell::new(false),
            active_touch_points: DOMRefCell::new(Vec::new()),
            dom_loading: Cell::new(Default::default()),
            dom_interactive: Cell::new(Default::default()),
            dom_content_loaded_event_start: Cell::new(Default::default()),
            dom_content_loaded_event_end: Cell::new(Default::default()),
            dom_complete: Cell::new(Default::default()),
            load_event_start: Cell::new(Default::default()),
            load_event_end: Cell::new(Default::default()),
            https_state: Cell::new(HttpsState::None),
            touchpad_pressure_phase: Cell::new(TouchpadPressurePhase::BeforeClick),
            origin: origin,
            referrer: referrer,
            referrer_policy: Cell::new(referrer_policy),
            target_element: MutNullableHeap::new(None),
            last_click_info: DOMRefCell::new(None),
        }
    }

    // https://dom.spec.whatwg.org/#dom-document
    pub fn Constructor(global: &GlobalScope) -> Fallible<Root<Document>> {
        let win = global.as_window();
        let doc = win.Document();
        let docloader = DocumentLoader::new(&*doc.loader());
        Ok(Document::new(win,
                         None,
                         None,
                         IsHTMLDocument::NonHTMLDocument,
                         None,
                         None,
                         DocumentSource::NotFromParser,
                         docloader,
                         None,
                         None))
    }

    pub fn new(window: &Window,
               browsing_context: Option<&BrowsingContext>,
               url: Option<Url>,
               doctype: IsHTMLDocument,
               content_type: Option<DOMString>,
               last_modified: Option<String>,
               source: DocumentSource,
               doc_loader: DocumentLoader,
               referrer: Option<String>,
               referrer_policy: Option<ReferrerPolicy>)
               -> Root<Document> {
        let document = reflect_dom_object(box Document::new_inherited(window,
                                                                      browsing_context,
                                                                      url,
                                                                      doctype,
                                                                      content_type,
                                                                      last_modified,
                                                                      source,
                                                                      doc_loader,
                                                                      referrer,
                                                                      referrer_policy),
                                          window,
                                          DocumentBinding::Wrap);
        {
            let node = document.upcast::<Node>();
            node.set_owner_doc(&document);
        }
        document
    }

    fn create_node_list<F: Fn(&Node) -> bool>(&self, callback: F) -> Root<NodeList> {
        let doc = self.GetDocumentElement();
        let maybe_node = doc.r().map(Castable::upcast::<Node>);
        let iter = maybe_node.iter()
                             .flat_map(|node| node.traverse_preorder())
                             .filter(|node| callback(&node));
        NodeList::new_simple_list(&self.window, iter)
    }

    fn get_html_element(&self) -> Option<Root<HTMLHtmlElement>> {
        self.GetDocumentElement().and_then(Root::downcast)
    }

    /// Returns the list of stylesheets associated with nodes in the document.
    pub fn stylesheets(&self) -> Vec<Arc<Stylesheet>> {
        {
            let mut stylesheets = self.stylesheets.borrow_mut();
            if stylesheets.is_none() {
                *stylesheets = Some(self.upcast::<Node>()
                    .traverse_preorder()
                    .filter_map(|node| {
                        if let Some(node) = node.downcast::<HTMLStyleElement>() {
                            node.get_stylesheet()
                        } else if let Some(node) = node.downcast::<HTMLLinkElement>() {
                            node.get_stylesheet()
                        } else if let Some(node) = node.downcast::<HTMLMetaElement>() {
                            node.get_stylesheet()
                        } else {
                            None
                        }.map(|stylesheet| StylesheetInDocument {
                            node: JS::from_ref(&*node),
                            stylesheet: stylesheet
                        })
                    })
                    .collect());
            };
        }
        self.stylesheets.borrow().as_ref().unwrap().iter()
                        .map(|s| s.stylesheet.clone())
                        .collect()
    }

    /// https://html.spec.whatwg.org/multipage/#appropriate-template-contents-owner-document
    pub fn appropriate_template_contents_owner_document(&self) -> Root<Document> {
        self.appropriate_template_contents_owner_document.or_init(|| {
            let doctype = if self.is_html_document {
                IsHTMLDocument::HTMLDocument
            } else {
                IsHTMLDocument::NonHTMLDocument
            };
            let new_doc = Document::new(self.window(),
                                        None,
                                        None,
                                        doctype,
                                        None,
                                        None,
                                        DocumentSource::NotFromParser,
                                        DocumentLoader::new(&self.loader()),
                                        None,
                                        None);
            new_doc.appropriate_template_contents_owner_document.set(Some(&new_doc));
            new_doc
        })
    }

    pub fn get_element_by_id(&self, id: &Atom) -> Option<Root<Element>> {
        self.id_map.borrow().get(&id).map(|ref elements| Root::from_ref(&*(*elements)[0]))
    }

    pub fn element_state_will_change(&self, el: &Element) {
        let mut map = self.modified_elements.borrow_mut();
        let snapshot = map.entry(JS::from_ref(el))
                          .or_insert_with(|| {
                              ElementSnapshot::new(el.html_element_in_html_document())
                          });
        if snapshot.state.is_none() {
            snapshot.state = Some(el.state());
        }
    }

    pub fn element_attr_will_change(&self, el: &Element) {
        let mut map = self.modified_elements.borrow_mut();
        let mut snapshot = map.entry(JS::from_ref(el))
                              .or_insert_with(|| {
                                  ElementSnapshot::new(el.html_element_in_html_document())
                              });
        if snapshot.attrs.is_none() {
            let attrs = el.attrs()
                          .iter()
                          .map(|attr| (attr.identifier().clone(), attr.value().clone()))
                          .collect();
            snapshot.attrs = Some(attrs);
        }
    }

    pub fn set_referrer_policy(&self, policy: Option<ReferrerPolicy>) {
        self.referrer_policy.set(policy);
    }

    //TODO - default still at no-referrer
    pub fn get_referrer_policy(&self) -> Option<ReferrerPolicy> {
        return self.referrer_policy.get();
    }

    pub fn set_target_element(&self, node: Option<&Element>) {
        if let Some(ref element) = self.target_element.get() {
            element.set_target_state(false);
        }

        self.target_element.set(node);

        if let Some(ref element) = self.target_element.get() {
            element.set_target_state(true);
        }

        self.window.reflow(ReflowGoal::ForDisplay,
                           ReflowQueryType::NoQuery,
                           ReflowReason::ElementStateChanged);
    }
}


impl Element {
    fn click_event_filter_by_disabled_state(&self) -> bool {
        let node = self.upcast::<Node>();
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) |
            // NodeTypeId::Element(ElementTypeId::HTMLKeygenElement) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement))
                if self.disabled_state() => true,
            _ => false,
        }
    }
}

impl DocumentMethods for Document {
    // https://drafts.csswg.org/cssom/#dom-document-stylesheets
    fn StyleSheets(&self) -> Root<StyleSheetList> {
        StyleSheetList::new(&self.window, JS::from_ref(&self))
    }

    // https://dom.spec.whatwg.org/#dom-document-implementation
    fn Implementation(&self) -> Root<DOMImplementation> {
        self.implementation.or_init(|| DOMImplementation::new(self))
    }

    // https://dom.spec.whatwg.org/#dom-document-url
    fn URL(&self) -> USVString {
        USVString(String::from(self.url().as_str()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-activeelement
    fn GetActiveElement(&self) -> Option<Root<Element>> {
        // TODO: Step 2.

        match self.get_focused_element() {
            Some(element) => Some(element),     // Step 3. and 4.
            None => match self.GetBody() {      // Step 5.
                Some(body) => Some(Root::upcast(body)),
                None => self.GetDocumentElement(),
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-hasfocus
    fn HasFocus(&self) -> bool {
        match self.browsing_context() {
            Some(browsing_context) => {
                // Step 2.
                let candidate = browsing_context.active_document();
                // Step 3.
                if &*candidate == self {
                    true
                } else {
                    false //TODO  Step 4.
                }
            }
            None => false,
        }
    }

    // https://html.spec.whatwg.org/multipage/#relaxing-the-same-origin-restriction
    fn Domain(&self) -> DOMString {
        // Step 1.
        if self.browsing_context().is_none() {
            return DOMString::new();
        }

        if let Some(host) = self.origin.host() {
            // Step 4.
            DOMString::from(host.to_string())
        } else {
            // Step 3.
            DOMString::new()
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-referrer
    fn Referrer(&self) -> DOMString {
        match self.referrer {
            Some(ref referrer) => DOMString::from(referrer.to_string()),
            None => DOMString::new()
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-documenturi
    fn DocumentURI(&self) -> USVString {
        self.URL()
    }

    // https://dom.spec.whatwg.org/#dom-document-compatmode
    fn CompatMode(&self) -> DOMString {
        DOMString::from(match self.quirks_mode.get() {
            LimitedQuirks | NoQuirks => "CSS1Compat",
            Quirks => "BackCompat",
        })
    }

    // https://dom.spec.whatwg.org/#dom-document-characterset
    fn CharacterSet(&self) -> DOMString {
        DOMString::from(match self.encoding.get().name() {
            "utf-8"         => "UTF-8",
            "ibm866"        => "IBM866",
            "iso-8859-2"    => "ISO-8859-2",
            "iso-8859-3"    => "ISO-8859-3",
            "iso-8859-4"    => "ISO-8859-4",
            "iso-8859-5"    => "ISO-8859-5",
            "iso-8859-6"    => "ISO-8859-6",
            "iso-8859-7"    => "ISO-8859-7",
            "iso-8859-8"    => "ISO-8859-8",
            "iso-8859-8-i"  => "ISO-8859-8-I",
            "iso-8859-10"   => "ISO-8859-10",
            "iso-8859-13"   => "ISO-8859-13",
            "iso-8859-14"   => "ISO-8859-14",
            "iso-8859-15"   => "ISO-8859-15",
            "iso-8859-16"   => "ISO-8859-16",
            "koi8-r"        => "KOI8-R",
            "koi8-u"        => "KOI8-U",
            "gbk"           => "GBK",
            "big5"          => "Big5",
            "euc-jp"        => "EUC-JP",
            "iso-2022-jp"   => "ISO-2022-JP",
            "shift_jis"     => "Shift_JIS",
            "euc-kr"        => "EUC-KR",
            "utf-16be"      => "UTF-16BE",
            "utf-16le"      => "UTF-16LE",
            name            => name
        })
    }

    // https://dom.spec.whatwg.org/#dom-document-charset
    fn Charset(&self) -> DOMString {
        self.CharacterSet()
    }

    // https://dom.spec.whatwg.org/#dom-document-inputencoding
    fn InputEncoding(&self) -> DOMString {
        self.CharacterSet()
    }

    // https://dom.spec.whatwg.org/#dom-document-content_type
    fn ContentType(&self) -> DOMString {
        self.content_type.clone()
    }

    // https://dom.spec.whatwg.org/#dom-document-doctype
    fn GetDoctype(&self) -> Option<Root<DocumentType>> {
        self.upcast::<Node>().children().filter_map(Root::downcast).next()
    }

    // https://dom.spec.whatwg.org/#dom-document-documentelement
    fn GetDocumentElement(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbytagname
    fn GetElementsByTagName(&self, qualified_name: DOMString) -> Root<HTMLCollection> {
        let qualified_name = LocalName::from(&*qualified_name);
        match self.tag_map.borrow_mut().entry(qualified_name.clone()) {
            Occupied(entry) => Root::from_ref(entry.get()),
            Vacant(entry) => {
                let result = HTMLCollection::by_qualified_name(
                    &self.window, self.upcast(), qualified_name);
                entry.insert(JS::from_ref(&*result));
                result
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbytagnamens
    fn GetElementsByTagNameNS(&self,
                              maybe_ns: Option<DOMString>,
                              tag_name: DOMString)
                              -> Root<HTMLCollection> {
        let ns = namespace_from_domstring(maybe_ns);
        let local = LocalName::from(tag_name);
        let qname = QualName::new(ns, local);
        match self.tagns_map.borrow_mut().entry(qname.clone()) {
            Occupied(entry) => Root::from_ref(entry.get()),
            Vacant(entry) => {
                let result = HTMLCollection::by_qual_tag_name(&self.window, self.upcast(), qname);
                entry.insert(JS::from_ref(&*result));
                result
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> Root<HTMLCollection> {
        let class_atoms: Vec<Atom> = split_html_space_chars(&classes)
                                         .map(Atom::from)
                                         .collect();
        match self.classes_map.borrow_mut().entry(class_atoms.clone()) {
            Occupied(entry) => Root::from_ref(entry.get()),
            Vacant(entry) => {
                let result = HTMLCollection::by_atomic_class_name(&self.window,
                                                                  self.upcast(),
                                                                  class_atoms);
                entry.insert(JS::from_ref(&*result));
                result
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(&self, id: DOMString) -> Option<Root<Element>> {
        self.get_element_by_id(&Atom::from(id))
    }

    // https://dom.spec.whatwg.org/#dom-document-createelement
    fn CreateElement(&self, mut local_name: DOMString) -> Fallible<Root<Element>> {
        if xml_name_type(&local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(Error::InvalidCharacter);
        }
        if self.is_html_document {
            local_name.make_ascii_lowercase();
        }
        let name = QualName::new(ns!(html), LocalName::from(local_name));
        Ok(Element::create(name, None, self, ElementCreator::ScriptCreated))
    }

    // https://dom.spec.whatwg.org/#dom-document-createelementns
    fn CreateElementNS(&self,
                       namespace: Option<DOMString>,
                       qualified_name: DOMString)
                       -> Fallible<Root<Element>> {
        let (namespace, prefix, local_name) = try!(validate_and_extract(namespace,
                                                                        &qualified_name));
        let name = QualName::new(namespace, local_name);
        Ok(Element::create(name, prefix, self, ElementCreator::ScriptCreated))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattribute
    fn CreateAttribute(&self, mut local_name: DOMString) -> Fallible<Root<Attr>> {
        if xml_name_type(&local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(Error::InvalidCharacter);
        }
        if self.is_html_document {
            local_name.make_ascii_lowercase();
        }
        let name = LocalName::from(local_name);
        let value = AttrValue::String("".to_owned());

        Ok(Attr::new(&self.window, name.clone(), value, name, ns!(), None, None))
    }

    // https://dom.spec.whatwg.org/#dom-document-createattributens
    fn CreateAttributeNS(&self,
                         namespace: Option<DOMString>,
                         qualified_name: DOMString)
                         -> Fallible<Root<Attr>> {
        let (namespace, prefix, local_name) = try!(validate_and_extract(namespace,
                                                                        &qualified_name));
        let value = AttrValue::String("".to_owned());
        let qualified_name = LocalName::from(qualified_name);
        Ok(Attr::new(&self.window,
                     local_name,
                     value,
                     qualified_name,
                     namespace,
                     prefix,
                     None))
    }

    // https://dom.spec.whatwg.org/#dom-document-createdocumentfragment
    fn CreateDocumentFragment(&self) -> Root<DocumentFragment> {
        DocumentFragment::new(self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createtextnode
    fn CreateTextNode(&self, data: DOMString) -> Root<Text> {
        Text::new(data, self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createcomment
    fn CreateComment(&self, data: DOMString) -> Root<Comment> {
        Comment::new(data, self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createprocessinginstruction
    fn CreateProcessingInstruction(&self,
                                   target: DOMString,
                                   data: DOMString)
                                   -> Fallible<Root<ProcessingInstruction>> {
        // Step 1.
        if xml_name_type(&target) == InvalidXMLName {
            return Err(Error::InvalidCharacter);
        }

        // Step 2.
        if data.contains("?>") {
            return Err(Error::InvalidCharacter);
        }

        // Step 3.
        Ok(ProcessingInstruction::new(target, data, self))
    }

    // https://dom.spec.whatwg.org/#dom-document-importnode
    fn ImportNode(&self, node: &Node, deep: bool) -> Fallible<Root<Node>> {
        // Step 1.
        if node.is::<Document>() {
            return Err(Error::NotSupported);
        }

        // Step 2.
        let clone_children = if deep {
            CloneChildrenFlag::CloneChildren
        } else {
            CloneChildrenFlag::DoNotCloneChildren
        };

        Ok(Node::clone(node, Some(self), clone_children))
    }

    // https://dom.spec.whatwg.org/#dom-document-adoptnode
    fn AdoptNode(&self, node: &Node) -> Fallible<Root<Node>> {
        // Step 1.
        if node.is::<Document>() {
            return Err(Error::NotSupported);
        }

        // Step 2.
        Node::adopt(node, self);

        // Step 3.
        Ok(Root::from_ref(node))
    }

    // https://dom.spec.whatwg.org/#dom-document-createevent
    fn CreateEvent(&self, mut interface: DOMString) -> Fallible<Root<Event>> {
        interface.make_ascii_lowercase();
        match &*interface {
            "uievents" | "uievent" =>
                Ok(Root::upcast(UIEvent::new_uninitialized(&self.window))),
            "mouseevents" | "mouseevent" =>
                Ok(Root::upcast(MouseEvent::new_uninitialized(&self.window))),
            "customevent" =>
                Ok(Root::upcast(CustomEvent::new_uninitialized(self.window.upcast()))),
            "htmlevents" | "events" | "event" | "svgevents" =>
                Ok(Event::new_uninitialized(&self.window.upcast())),
            "keyboardevent" =>
                Ok(Root::upcast(KeyboardEvent::new_uninitialized(&self.window))),
            "messageevent" =>
                Ok(Root::upcast(MessageEvent::new_uninitialized(self.window.upcast()))),
            "touchevent" =>
                Ok(Root::upcast(
                    TouchEvent::new_uninitialized(&self.window,
                        &TouchList::new(&self.window, &[]),
                        &TouchList::new(&self.window, &[]),
                        &TouchList::new(&self.window, &[]),
                    )
                )),
            "webglcontextevent" =>
                Ok(Root::upcast(WebGLContextEvent::new_uninitialized(self.window.upcast()))),
            "storageevent" => {
                let USVString(url) = self.URL();
                Ok(Root::upcast(StorageEvent::new_uninitialized(&self.window, DOMString::from(url))))
            },
            "progressevent" =>
                Ok(Root::upcast(ProgressEvent::new_uninitialized(self.window.upcast()))),
            "focusevent" =>
                Ok(Root::upcast(FocusEvent::new_uninitialized(self.window.upcast()))),
            "errorevent" =>
                Ok(Root::upcast(ErrorEvent::new_uninitialized(self.window.upcast()))),
            "closeevent" =>
                Ok(Root::upcast(CloseEvent::new_uninitialized(self.window.upcast()))),
            "popstateevent" =>
                Ok(Root::upcast(PopStateEvent::new_uninitialized(self.window.upcast()))),
            "hashchangeevent" =>
                Ok(Root::upcast(HashChangeEvent::new_uninitialized(&self.window.upcast()))),
            "pagetransitionevent" =>
                Ok(Root::upcast(PageTransitionEvent::new_uninitialized(self.window.upcast()))),
            _ =>
                Err(Error::NotSupported),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-lastmodified
    fn LastModified(&self) -> DOMString {
        match self.last_modified {
            Some(ref t) => DOMString::from(t.clone()),
            None => DOMString::from(time::now().strftime("%m/%d/%Y %H:%M:%S").unwrap().to_string()),
        }
    }

    // https://dom.spec.whatwg.org/#dom-document-createrange
    fn CreateRange(&self) -> Root<Range> {
        Range::new_with_doc(self)
    }

    // https://dom.spec.whatwg.org/#dom-document-createnodeiteratorroot-whattoshow-filter
    fn CreateNodeIterator(&self,
                          root: &Node,
                          what_to_show: u32,
                          filter: Option<Rc<NodeFilter>>)
                          -> Root<NodeIterator> {
        NodeIterator::new(self, root, what_to_show, filter)
    }

    // https://w3c.github.io/touch-events/#idl-def-Document
    fn CreateTouch(&self,
                   window: &Window,
                   target: &EventTarget,
                   identifier: i32,
                   page_x: Finite<f64>,
                   page_y: Finite<f64>,
                   screen_x: Finite<f64>,
                   screen_y: Finite<f64>)
                   -> Root<Touch> {
        let client_x = Finite::wrap(*page_x - window.PageXOffset() as f64);
        let client_y = Finite::wrap(*page_y - window.PageYOffset() as f64);
        Touch::new(window,
                   identifier,
                   target,
                   screen_x,
                   screen_y,
                   client_x,
                   client_y,
                   page_x,
                   page_y)
    }

    // https://w3c.github.io/touch-events/#idl-def-document-createtouchlist(touch...)
    fn CreateTouchList(&self, touches: &[&Touch]) -> Root<TouchList> {
        TouchList::new(&self.window, &touches)
    }

    // https://dom.spec.whatwg.org/#dom-document-createtreewalker
    fn CreateTreeWalker(&self,
                        root: &Node,
                        what_to_show: u32,
                        filter: Option<Rc<NodeFilter>>)
                        -> Root<TreeWalker> {
        TreeWalker::new(self, root, what_to_show, filter)
    }

    // https://html.spec.whatwg.org/multipage/#document.title
    fn Title(&self) -> DOMString {
        let title = self.GetDocumentElement().and_then(|root| {
            if root.namespace() == &ns!(svg) && root.local_name() == &local_name!("svg") {
                // Step 1.
                root.upcast::<Node>()
                    .child_elements()
                    .find(|node| {
                        node.namespace() == &ns!(svg) && node.local_name() == &local_name!("title")
                    })
                    .map(Root::upcast::<Node>)
            } else {
                // Step 2.
                root.upcast::<Node>()
                    .traverse_preorder()
                    .find(|node| node.is::<HTMLTitleElement>())
            }
        });

        match title {
            None => DOMString::new(),
            Some(ref title) => {
                // Steps 3-4.
                let value = Node::collect_text_contents(title.children());
                DOMString::from(str_join(split_html_space_chars(&value), " "))
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#document.title
    fn SetTitle(&self, title: DOMString) {
        let root = match self.GetDocumentElement() {
            Some(root) => root,
            None => return,
        };

        let elem = if root.namespace() == &ns!(svg) && root.local_name() == &local_name!("svg") {
            let elem = root.upcast::<Node>().child_elements().find(|node| {
                node.namespace() == &ns!(svg) && node.local_name() == &local_name!("title")
            });
            match elem {
                Some(elem) => Root::upcast::<Node>(elem),
                None => {
                    let name = QualName::new(ns!(svg), local_name!("title"));
                    let elem = Element::create(name, None, self, ElementCreator::ScriptCreated);
                    let parent = root.upcast::<Node>();
                    let child = elem.upcast::<Node>();
                    parent.InsertBefore(child, parent.GetFirstChild().r())
                          .unwrap()
                }
            }
        } else if root.namespace() == &ns!(html) {
            let elem = root.upcast::<Node>()
                           .traverse_preorder()
                           .find(|node| node.is::<HTMLTitleElement>());
            match elem {
                Some(elem) => elem,
                None => {
                    match self.GetHead() {
                        Some(head) => {
                            let name = QualName::new(ns!(html), local_name!("title"));
                            let elem = Element::create(name,
                                                       None,
                                                       self,
                                                       ElementCreator::ScriptCreated);
                            head.upcast::<Node>()
                                .AppendChild(elem.upcast())
                                .unwrap()
                        },
                        None => return,
                    }
                }
            }
        } else {
            return;
        };

        elem.SetTextContent(Some(title));
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-head
    fn GetHead(&self) -> Option<Root<HTMLHeadElement>> {
        self.get_html_element()
            .and_then(|root| root.upcast::<Node>().children().filter_map(Root::downcast).next())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-currentscript
    fn GetCurrentScript(&self) -> Option<Root<HTMLScriptElement>> {
        self.current_script.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-body
    fn GetBody(&self) -> Option<Root<HTMLElement>> {
        self.get_html_element().and_then(|root| {
            let node = root.upcast::<Node>();
            node.children().find(|child| {
                match child.type_id() {
                    NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) |
                    NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFrameSetElement)) => true,
                    _ => false
                }
            }).map(|node| Root::downcast(node).unwrap())
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-body
    fn SetBody(&self, new_body: Option<&HTMLElement>) -> ErrorResult {
        // Step 1.
        let new_body = match new_body {
            Some(new_body) => new_body,
            None => return Err(Error::HierarchyRequest),
        };

        let node = new_body.upcast::<Node>();
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFrameSetElement)) => {}
            _ => return Err(Error::HierarchyRequest),
        }

        // Step 2.
        let old_body = self.GetBody();
        if old_body.r() == Some(new_body) {
            return Ok(());
        }

        match (self.get_html_element(), &old_body) {
            // Step 3.
            (Some(ref root), &Some(ref child)) => {
                let root = root.upcast::<Node>();
                root.ReplaceChild(new_body.upcast(), child.upcast()).unwrap();
            },

            // Step 4.
            (None, _) => return Err(Error::HierarchyRequest),

            // Step 5.
            (Some(ref root), &None) => {
                let root = root.upcast::<Node>();
                root.AppendChild(new_body.upcast()).unwrap();
            }
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-getelementsbyname
    fn GetElementsByName(&self, name: DOMString) -> Root<NodeList> {
        self.create_node_list(|node| {
            let element = match node.downcast::<Element>() {
                Some(element) => element,
                None => return false,
            };
            if element.namespace() != &ns!(html) {
                return false;
            }
            element.get_attribute(&ns!(), &local_name!("name"))
                   .map_or(false, |attr| &**attr.value() == &*name)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-images
    fn Images(&self) -> Root<HTMLCollection> {
        self.images.or_init(|| {
            let filter = box ImagesFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-embeds
    fn Embeds(&self) -> Root<HTMLCollection> {
        self.embeds.or_init(|| {
            let filter = box EmbedsFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-plugins
    fn Plugins(&self) -> Root<HTMLCollection> {
        self.Embeds()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-links
    fn Links(&self) -> Root<HTMLCollection> {
        self.links.or_init(|| {
            let filter = box LinksFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-forms
    fn Forms(&self) -> Root<HTMLCollection> {
        self.forms.or_init(|| {
            let filter = box FormsFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-scripts
    fn Scripts(&self) -> Root<HTMLCollection> {
        self.scripts.or_init(|| {
            let filter = box ScriptsFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-anchors
    fn Anchors(&self) -> Root<HTMLCollection> {
        self.anchors.or_init(|| {
            let filter = box AnchorsFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-applets
    fn Applets(&self) -> Root<HTMLCollection> {
        // FIXME: This should be return OBJECT elements containing applets.
        self.applets.or_init(|| {
            let filter = box AppletsFilter;
            HTMLCollection::create(&self.window, self.upcast(), filter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-location
    fn GetLocation(&self) -> Option<Root<Location>> {
        self.browsing_context().map(|_| self.location.or_init(|| Location::new(&self.window)))
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self) -> Root<HTMLCollection> {
        HTMLCollection::children(&self.window, self.upcast())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().rev_children().filter_map(Root::downcast).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    fn ChildElementCount(&self) -> u32 {
        self.upcast::<Node>().child_elements().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn Prepend(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().prepend(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().append(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Root<Element>>> {
        let root = self.upcast::<Node>();
        root.query_selector(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<Root<NodeList>> {
        let root = self.upcast::<Node>();
        root.query_selector_all(selectors)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-readystate
    fn ReadyState(&self) -> DocumentReadyState {
        self.ready_state.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-defaultview
    fn GetDefaultView(&self) -> Option<Root<Window>> {
        if self.browsing_context.is_none() {
            None
        } else {
            Some(Root::from_ref(&*self.window))
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-cookie
    fn GetCookie(&self) -> Fallible<DOMString> {
        if self.is_cookie_averse() {
            return Ok(DOMString::new());
        }

        if !self.origin.is_scheme_host_port_tuple() {
            return Err(Error::Security);
        }

        let url = self.url();
        let (tx, rx) = ipc::channel().unwrap();
        let _ = self.window
            .upcast::<GlobalScope>()
            .resource_threads()
            .send(GetCookiesForUrl((*url).clone(), tx, NonHTTP));
        let cookies = rx.recv().unwrap();
        Ok(cookies.map_or(DOMString::new(), DOMString::from))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-cookie
    fn SetCookie(&self, cookie: DOMString) -> ErrorResult {
        if self.is_cookie_averse() {
            return Ok(());
        }

        if !self.origin.is_scheme_host_port_tuple() {
            return Err(Error::Security);
        }

        let url = self.url();
        let _ = self.window
                    .upcast::<GlobalScope>()
                    .resource_threads()
                    .send(SetCookiesForUrl((*url).clone(), String::from(cookie), NonHTTP));
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-bgcolor
    fn BgColor(&self) -> DOMString {
        self.get_body_attribute(&local_name!("bgcolor"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-bgcolor
    fn SetBgColor(&self, value: DOMString) {
        self.set_body_attribute(&local_name!("bgcolor"), value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-fgcolor
    fn FgColor(&self) -> DOMString {
        self.get_body_attribute(&local_name!("text"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-fgcolor
    fn SetFgColor(&self, value: DOMString) {
        self.set_body_attribute(&local_name!("text"), value)
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:dom-document-nameditem-filter
    fn NamedGetter(&self, _cx: *mut JSContext, name: DOMString) -> Option<NonZero<*mut JSObject>> {
        #[derive(JSTraceable, HeapSizeOf)]
        struct NamedElementFilter {
            name: Atom,
        }
        impl CollectionFilter for NamedElementFilter {
            fn filter(&self, elem: &Element, _root: &Node) -> bool {
                filter_by_name(&self.name, elem.upcast())
            }
        }
        // https://html.spec.whatwg.org/multipage/#dom-document-nameditem-filter
        fn filter_by_name(name: &Atom, node: &Node) -> bool {
            let html_elem_type = match node.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(type_)) => type_,
                _ => return false,
            };
            let elem = match node.downcast::<Element>() {
                Some(elem) => elem,
                None => return false,
            };
            match html_elem_type {
                HTMLElementTypeId::HTMLAppletElement => {
                    match elem.get_attribute(&ns!(), &local_name!("name")) {
                        Some(ref attr) if attr.value().as_atom() == name => true,
                        _ => {
                            match elem.get_attribute(&ns!(), &local_name!("id")) {
                                Some(ref attr) => attr.value().as_atom() == name,
                                None => false,
                            }
                        },
                    }
                },
                HTMLElementTypeId::HTMLFormElement => {
                    match elem.get_attribute(&ns!(), &local_name!("name")) {
                        Some(ref attr) => attr.value().as_atom() == name,
                        None => false,
                    }
                },
                HTMLElementTypeId::HTMLImageElement => {
                    match elem.get_attribute(&ns!(), &local_name!("name")) {
                        Some(ref attr) => {
                            if attr.value().as_atom() == name {
                                true
                            } else {
                                match elem.get_attribute(&ns!(), &local_name!("id")) {
                                    Some(ref attr) => attr.value().as_atom() == name,
                                    None => false,
                                }
                            }
                        },
                        None => false,
                    }
                },
                // TODO: Handle <embed>, <iframe> and <object>.
                _ => false,
            }
        }
        let name = Atom::from(name);
        let root = self.upcast::<Node>();
        {
            // Step 1.
            let mut elements = root.traverse_preorder()
                                   .filter(|node| filter_by_name(&name, &node))
                                   .peekable();
            if let Some(first) = elements.next() {
                if elements.peek().is_none() {
                    // TODO: Step 2.
                    // Step 3.
                    return unsafe {
                        Some(NonZero::new(first.reflector().get_jsobject().get()))
                    };
                }
            } else {
                return None;
            }
        }
        // Step 4.
        let filter = NamedElementFilter {
            name: name,
        };
        let collection = HTMLCollection::create(self.window(), root, box filter);
        unsafe {
            Some(NonZero::new(collection.reflector().get_jsobject().get()))
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        // FIXME: unimplemented (https://github.com/servo/servo/issues/7273)
        vec![]
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-clear
    fn Clear(&self) {
        // This method intentionally does nothing
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-captureevents
    fn CaptureEvents(&self) {
        // This method intentionally does nothing
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-releaseevents
    fn ReleaseEvents(&self) {
        // This method intentionally does nothing
    }

    // https://html.spec.whatwg.org/multipage/#globaleventhandlers
    global_event_handlers!();

    // https://html.spec.whatwg.org/multipage/#handler-onreadystatechange
    event_handler!(readystatechange, GetOnreadystatechange, SetOnreadystatechange);

    #[allow(unsafe_code)]
    // https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint
    fn ElementFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Option<Root<Element>> {
        let x = *x as f32;
        let y = *y as f32;
        let point = &Point2D::new(x, y);
        let window = window_from_node(self);
        let viewport = window.window_size().unwrap().visible_viewport;

        if self.browsing_context().is_none() {
            return None;
        }

        if x < 0.0 || y < 0.0 || x > viewport.width || y > viewport.height {
            return None;
        }

        match self.window.hit_test_query(*point, false) {
            Some(untrusted_node_address) => {
                let js_runtime = unsafe { JS_GetRuntime(window.get_cx()) };

                let node = node::from_untrusted_node_address(js_runtime, untrusted_node_address);
                let parent_node = node.GetParentNode().unwrap();
                let element_ref = node.downcast::<Element>().unwrap_or_else(|| {
                    parent_node.downcast::<Element>().unwrap()
                });

                Some(Root::from_ref(element_ref))
            },
            None => self.GetDocumentElement()
        }
    }

    #[allow(unsafe_code)]
    // https://drafts.csswg.org/cssom-view/#dom-document-elementsfrompoint
    fn ElementsFromPoint(&self, x: Finite<f64>, y: Finite<f64>) -> Vec<Root<Element>> {
        let x = *x as f32;
        let y = *y as f32;
        let point = &Point2D::new(x, y);
        let window = window_from_node(self);
        let viewport = window.window_size().unwrap().visible_viewport;

        if self.browsing_context().is_none() {
            return vec!();
        }

        // Step 2
        if x < 0.0 || y < 0.0 || x > viewport.width || y > viewport.height {
            return vec!();
        }

        let js_runtime = unsafe { JS_GetRuntime(window.get_cx()) };

        // Step 1 and Step 3
        let mut elements: Vec<Root<Element>> = self.nodes_from_point(point).iter()
            .flat_map(|&untrusted_node_address| {
                let node = node::from_untrusted_node_address(js_runtime, untrusted_node_address);
                Root::downcast::<Element>(node)
        }).collect();

        // Step 4
        if let Some(root_element) = self.GetDocumentElement() {
            if elements.last() != Some(&root_element) {
                elements.push(root_element);
            }
        }

        // Step 5
        elements
    }

    // https://html.spec.whatwg.org/multipage/#documentandelementeventhandlers
    document_and_element_event_handlers!();
}

fn update_with_current_time_ms(marker: &Cell<u64>) {
    if marker.get() == Default::default() {
        let time = time::get_time();
        let current_time_ms = time.sec * 1000 + time.nsec as i64 / 1000000;
        marker.set(current_time_ms as u64);
    }
}

/// https://w3c.github.io/webappsec-referrer-policy/#determine-policy-for-token
pub fn determine_policy_for_token(token: &str) -> Option<ReferrerPolicy> {
    let lower = token.to_lowercase();
    return match lower.as_ref() {
        "never" | "no-referrer" => Some(ReferrerPolicy::NoReferrer),
        "default" | "no-referrer-when-downgrade" => Some(ReferrerPolicy::NoReferrerWhenDowngrade),
        "origin" => Some(ReferrerPolicy::Origin),
        "same-origin" => Some(ReferrerPolicy::SameOrigin),
        "strict-origin" => Some(ReferrerPolicy::StrictOrigin),
        "strict-origin-when-cross-origin" => Some(ReferrerPolicy::StrictOriginWhenCrossOrigin),
        "origin-when-cross-origin" => Some(ReferrerPolicy::OriginWhenCrossOrigin),
        "always" | "unsafe-url" => Some(ReferrerPolicy::UnsafeUrl),
        "" => Some(ReferrerPolicy::NoReferrer),
        _ => None,
    }
}

pub struct DocumentProgressHandler {
    addr: Trusted<Document>
}

impl DocumentProgressHandler {
     pub fn new(addr: Trusted<Document>) -> DocumentProgressHandler {
        DocumentProgressHandler {
            addr: addr
        }
    }

    fn set_ready_state_complete(&self) {
        let document = self.addr.root();
        document.set_ready_state(DocumentReadyState::Complete);
    }

    fn dispatch_load(&self) {
        let document = self.addr.root();
        let window = document.window();
        let event = Event::new(window.upcast(),
                               atom!("load"),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        let wintarget = window.upcast::<EventTarget>();
        event.set_trusted(true);

        // http://w3c.github.io/navigation-timing/#widl-PerformanceNavigationTiming-loadEventStart
        update_with_current_time_ms(&document.load_event_start);

        debug!("About to dispatch load for {:?}", document.url());
        let _ = wintarget.dispatch_event_with_target(document.upcast(), &event);

        // http://w3c.github.io/navigation-timing/#widl-PerformanceNavigationTiming-loadEventEnd
        update_with_current_time_ms(&document.load_event_end);


        window.reflow(ReflowGoal::ForDisplay,
                      ReflowQueryType::NoQuery,
                      ReflowReason::DocumentLoaded);

        document.notify_constellation_load();
    }
}

impl Runnable for DocumentProgressHandler {
    fn name(&self) -> &'static str { "DocumentProgressHandler" }

    fn handler(self: Box<DocumentProgressHandler>) {
        let document = self.addr.root();
        let window = document.window();
        if window.is_alive() {
            self.set_ready_state_complete();
            self.dispatch_load();
        }
    }
}

/// Specifies the type of focus event that is sent to a pipeline
#[derive(Copy, Clone, PartialEq)]
pub enum FocusType {
    Element,    // The first focus message - focus the element itself
    Parent,     // Focusing a parent element (an iframe)
}

/// Focus events
pub enum FocusEventType {
    Focus,      // Element gained focus. Doesn't bubble.
    Blur,       // Element lost focus. Doesn't bubble.
}
