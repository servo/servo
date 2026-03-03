/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The implementation of the DOM.
//!
//! The DOM is comprised of interfaces (defined by specifications using
//! [WebIDL](https://heycam.github.io/webidl/)) that are implemented as Rust
//! structs in submodules of this module. Its implementation is documented
//! below.
//!
//! A DOM object and its reflector
//! ==============================
//!
//! The implementation of an interface `Foo` in Servo's DOM involves two
//! related but distinct objects:
//!
//! * the **DOM object**: an instance of the Rust struct `dom::foo::Foo`
//!   (marked with the `#[dom_struct]` attribute) on the Rust heap;
//! * the **reflector**: a `JSObject` allocated by SpiderMonkey, that owns the
//!   DOM object.
//!
//! Memory management
//! =================
//!
//! Reflectors of DOM objects, and thus the DOM objects themselves, are managed
//! by the SpiderMonkey Garbage Collector. Thus, keeping alive a DOM object
//! is done through its reflector.
//!
//! For more information, see:
//!
//! * rooting pointers on the stack:
//!   the [`Root`](bindings/root/struct.Root.html) smart pointer;
//! * tracing pointers in member fields: the [`Dom`](bindings/root/struct.Dom.html),
//!   [`MutNullableDom`](bindings/root/struct.MutNullableDom.html) and
//!   [`MutDom`](bindings/root/struct.MutDom.html) smart pointers and
//!   [the tracing implementation](bindings/trace/index.html);
//! * rooting pointers from across thread boundaries or in channels: the
//!   [`Trusted`](bindings/refcounted/struct.Trusted.html) smart pointer;
//!
//! Inheritance
//! ===========
//!
//! Rust does not support struct inheritance, as would be used for the
//! object-oriented DOM APIs. To work around this issue, Servo stores an
//! instance of the superclass in the first field of its subclasses. (Note that
//! it is stored by value, rather than in a smart pointer such as `Dom<T>`.)
//!
//! This implies that a pointer to an object can safely be cast to a pointer
//! to all its classes.
//!
//! This invariant is enforced by the lint in
//! `plugins::lints::inheritance_integrity`.
//!
//! Interfaces which either derive from or are derived by other interfaces
//! implement the `Castable` trait, which provides three methods `is::<T>()`,
//! `downcast::<T>()` and `upcast::<T>()` to cast across the type hierarchy
//! and check whether a given instance is of a given type.
//!
//! ```ignore
//! use dom::bindings::inheritance::Castable;
//! use dom::element::Element;
//! use dom::htmlelement::HTMLElement;
//! use dom::htmlinputelement::HTMLInputElement;
//!
//! if let Some(elem) = node.downcast::<Element> {
//!     if elem.is::<HTMLInputElement>() {
//!         return elem.upcast::<HTMLElement>();
//!     }
//! }
//! ```
//!
//! Furthermore, when discriminating a given instance against multiple
//! interface types, code generation provides a convenient TypeId enum
//! which can be used to write `match` expressions instead of multiple
//! calls to `Castable::is::<T>`. The `type_id()` method of an instance is
//! provided by the farthest interface it derives from, e.g. `EventTarget`
//! for `HTMLMediaElement`. For convenience, that method is also provided
//! on the `Node` interface to avoid unnecessary upcasts to `EventTarget`.
//!
//! ```ignore
//! use dom::bindings::inheritance::{EventTargetTypeId, NodeTypeId};
//!
//! match *node.type_id() {
//!     EventTargetTypeId::Node(NodeTypeId::CharacterData(_)) => ...,
//!     EventTargetTypeId::Node(NodeTypeId::Element(_)) => ...,
//!     ...,
//! }
//! ```
//!
//! Construction
//! ============
//!
//! DOM objects of type `T` in Servo have two constructors:
//!
//! * a `T::new_inherited` static method that returns a plain `T`, and
//! * a `T::new` static method that returns `DomRoot<T>`.
//!
//! (The result of either method can be wrapped in `Result`, if that is
//! appropriate for the type in question.)
//!
//! The latter calls the former, boxes the result, and creates a reflector
//! corresponding to it by calling `dom::bindings::utils::reflect_dom_object`
//! (which yields ownership of the object to the SpiderMonkey Garbage Collector).
//! This is the API to use when creating a DOM object.
//!
//! The former should only be called by the latter, and by subclasses'
//! `new_inherited` methods.
//!
//! DOM object constructors in JavaScript correspond to a `T::Constructor`
//! static method. This method is always fallible.
//!
//! Destruction
//! ===========
//!
//! When the SpiderMonkey Garbage Collector discovers that the reflector of a
//! DOM object is garbage, it calls the reflector's finalization hook. This
//! function deletes the reflector's DOM object, calling its destructor in the
//! process.
//!
//! Mutability and aliasing
//! =======================
//!
//! Reflectors are JavaScript objects, and as such can be freely aliased. As
//! Rust does not allow mutable aliasing, mutable borrows of DOM objects are
//! not allowed. In particular, any mutable fields use `Cell` or `DomRefCell`
//! to manage their mutability.
//!
//! `Reflector` and `DomObject`
//! =============================
//!
//! Every DOM object has a `Reflector` as its first (transitive) member field.
//! This contains a `*mut JSObject` that points to its reflector.
//!
//! The `FooBinding::Wrap` function creates the reflector, stores a pointer to
//! the DOM object in the reflector, and initializes the pointer to the reflector
//! in the `Reflector` field.
//!
//! The `DomObject` trait provides a `reflector()` method that returns the
//! DOM object's `Reflector`. It is implemented automatically for DOM structs
//! through the `#[dom_struct]` attribute.
//!
//! Implementing methods for a DOM object
//! =====================================
//!
//! * `dom::bindings::codegen::Bindings::FooBinding::FooMethods` for methods
//!   defined through IDL;
//! * `&self` public methods for public helpers;
//! * `&self` methods for private helpers.
//!
//! Accessing fields of a DOM object
//! ================================
//!
//! All fields of DOM objects are private; accessing them from outside their
//! module is done through explicit getter or setter methods.
//!
//! Inheritance and casting
//! =======================
//!
//! All DOM interfaces part of an inheritance chain (i.e. interfaces
//! that derive others or are derived from) implement the trait `Castable`
//! which provides both downcast and upcasts.
//!
//! ```ignore
//! # use script::dom::bindings::inheritance::Castable;
//! # use script::dom::element::Element;
//! # use script::dom::node::Node;
//! # use script::dom::htmlelement::HTMLElement;
//! fn f(element: &Element) {
//!     let base = element.upcast::<Node>();
//!     let derived = element.downcast::<HTMLElement>().unwrap();
//! }
//! ```
//!
//! Adding a new DOM interface
//! ==========================
//!
//! Adding a new interface `Foo` requires at least the following:
//!
//! * adding the new IDL file at `components/script/dom/webidls/Foo.webidl`;
//! * creating `components/script/dom/foo.rs`;
//! * listing `foo.rs` in `components/script/dom/mod.rs`;
//! * defining the DOM struct `Foo` with a `#[dom_struct]` attribute, a
//!   superclass or `Reflector` member, and other members as appropriate;
//! * implementing the
//!   `dom::bindings::codegen::Bindings::FooBinding::FooMethods` trait for
//!   `Foo`;
//! * adding/updating the match arm in create_element in
//!   `components/script/dom/create.rs` (only applicable to new types inheriting
//!   from `HTMLElement`)
//!
//! More information is available in the [bindings module](bindings/index.html).
//!
//! Accessing DOM objects from layout
//! =================================
//!
//! Layout code can access the DOM through the
//! [`LayoutDom`](bindings/root/struct.LayoutDom.html) smart pointer. This does not
//! keep the DOM object alive; we ensure that no DOM code (Garbage Collection
//! in particular) runs while layout is accessing the DOM.
//!
//! Methods accessible to layout are implemented on `LayoutDom<Foo>` using
//! `LayoutFooHelpers` traits.

#[macro_use]
pub(crate) mod macros;

pub(crate) mod types {
    include!(concat!(env!("OUT_DIR"), "/InterfaceTypes.rs"));
}

pub(crate) mod abortcontroller;
pub(crate) mod abortsignal;
#[expect(dead_code)]
pub(crate) mod abstractrange;
pub(crate) mod activation;
pub(crate) mod animationevent;
pub(crate) mod attr;
pub(crate) mod audio;
pub(crate) use self::audio::*;
pub(crate) mod beforeunloadevent;
pub(crate) mod bindings;
pub(crate) mod blob;
#[cfg(feature = "bluetooth")]
pub(crate) mod bluetooth;
#[cfg(feature = "bluetooth")]
pub(crate) use self::bluetooth::*;
pub(crate) mod broadcastchannel;
mod canvas;
pub(crate) use self::canvas::*;

pub(crate) mod cdatasection;
pub(crate) mod characterdata;
pub(crate) mod client;
pub(crate) mod clipboard;
pub(crate) mod clipboardevent;
pub(crate) mod clipboarditem;
pub(crate) mod closeevent;
pub(crate) mod commandevent;
pub(crate) mod comment;
pub(crate) mod compositionevent;
pub(crate) mod console;
pub(crate) mod cookiestore;
mod create;
pub(crate) mod credentialmanagement;
pub(crate) use self::credentialmanagement::*;
pub(crate) mod crypto;
pub(crate) mod cryptokey;
pub(crate) mod css;
pub(crate) use self::css::*;
pub(crate) mod customelementregistry;
pub(crate) mod customevent;
pub(crate) mod customstateset;
pub(crate) mod datatransfer;
pub(crate) mod datatransferitem;
pub(crate) mod datatransferitemlist;
pub(crate) mod debuggeradddebuggeeevent;
pub(crate) mod debuggerclearbreakpointevent;
pub(crate) mod debuggerevalevent;
pub(crate) mod debuggergetpossiblebreakpointsevent;
pub(crate) mod debuggerglobalscope;
pub(crate) mod debuggerinterruptevent;
pub(crate) mod debuggerresumeevent;
pub(crate) mod debuggersetbreakpointevent;
pub(crate) mod dissimilaroriginlocation;
pub(crate) mod dissimilaroriginwindow;
#[expect(dead_code)]
pub(crate) mod document;
mod document_embedder_controls;
pub(crate) mod document_event_handler;
pub(crate) mod documentfragment;
pub(crate) mod documentorshadowroot;
pub(crate) mod documenttype;
pub(crate) mod domexception;
pub(crate) mod domimplementation;
pub(crate) mod dommatrix;
pub(crate) mod dommatrixreadonly;
pub(crate) mod domparser;
pub(crate) mod dompoint;
pub(crate) mod dompointreadonly;
pub(crate) mod domquad;
pub(crate) mod domrect;
pub(crate) mod domrectlist;
pub(crate) mod domrectreadonly;
pub(crate) mod domstringlist;
pub(crate) mod domstringmap;
pub(crate) mod domtokenlist;
#[expect(dead_code)]
pub(crate) mod element;
pub(crate) mod elementinternals;
pub(crate) mod errorevent;
pub(crate) mod event;
pub(crate) mod eventsource;
pub(crate) mod eventtarget;
pub(crate) mod execcommand;
pub(crate) mod extendableevent;
pub(crate) mod extendablemessageevent;
pub(crate) mod fetchlaterresult;
pub(crate) mod file;
pub(crate) mod filelist;
pub(crate) mod filereader;
pub(crate) mod filereadersync;
pub(crate) mod focusevent;
pub(crate) mod formdata;
pub(crate) mod formdataevent;
#[cfg(feature = "gamepad")]
pub(crate) mod gamepad;
#[cfg(feature = "gamepad")]
pub(crate) use self::gamepad::*;
pub(crate) mod geolocation;
pub(crate) use self::geolocation::*;
pub(crate) mod global_scope_script_execution;
pub(crate) mod globalscope;
pub(crate) mod hashchangeevent;
pub(crate) mod headers;
pub(crate) mod history;
pub(crate) mod html;
pub(crate) use self::html::*;
pub(crate) mod indexeddb;
pub(crate) use self::indexeddb::*;
pub(crate) mod inputevent;
pub(crate) mod intersectionobserver;
pub(crate) mod intersectionobserverentry;
pub(crate) mod keyboardevent;
pub(crate) mod location;
pub(crate) mod media;
pub(crate) use self::media::*;
pub(crate) mod messagechannel;
pub(crate) mod messageevent;
#[expect(dead_code)]
pub(crate) mod messageport;
pub(crate) mod mimetype;
pub(crate) mod mimetypearray;
pub(crate) mod mouseevent;
pub(crate) mod mutationobserver;
pub(crate) mod mutationrecord;
pub(crate) mod namednodemap;
pub(crate) mod navigationpreloadmanager;
pub(crate) mod navigator;
pub(crate) mod navigatorinfo;
#[expect(dead_code)]
pub(crate) mod node;
pub(crate) mod nodeiterator;
#[expect(dead_code)]
pub(crate) mod nodelist;
pub(crate) mod notification;
pub(crate) mod origin;
pub(crate) mod pagetransitionevent;
pub(crate) mod paintsize;
pub(crate) mod paintworkletglobalscope;
pub(crate) mod performance;
pub(crate) use self::performance::*;
pub(crate) mod permissions;
pub(crate) mod permissionstatus;
pub(crate) mod pipelineid;
pub(crate) mod plugin;
pub(crate) mod pluginarray;
#[expect(dead_code)]
pub(crate) mod pointerevent;
pub(crate) mod popstateevent;
pub(crate) mod processinginstruction;
pub(crate) mod processingoptions;
pub(crate) mod progressevent;
#[expect(dead_code)]
pub(crate) mod promise;
pub(crate) mod promisenativehandler;
pub(crate) mod promiserejectionevent;
pub(crate) mod quotaexceedederror;
pub(crate) mod radionodelist;
pub(crate) mod range;
pub(crate) mod raredata;
#[allow(dead_code)]
pub(crate) mod reportingendpoint;
pub(crate) mod reportingobserver;
pub(crate) mod request;
pub(crate) mod resizeobserver;
pub(crate) mod resizeobserverentry;
pub(crate) mod resizeobserversize;
pub(crate) mod response;
pub(crate) mod screen;
mod scrolling_box;
pub(crate) mod security;
pub(crate) use self::security::*;
pub(crate) mod selection;
pub(crate) mod servointernals;
#[expect(dead_code)]
pub(crate) mod servoparser;
pub(crate) mod shadowroot;
pub(crate) mod staticrange;
pub(crate) mod storage;
pub(crate) mod storageevent;
pub(crate) mod stream;
pub(crate) use self::stream::*;
pub(crate) mod submitevent;
pub(crate) mod subtlecrypto;
pub(crate) mod svg;
pub(crate) use self::svg::*;
#[cfg(feature = "testbinding")]
mod testing;
#[cfg(feature = "testbinding")]
pub(crate) use self::testing::*;
pub(crate) mod text;
pub(crate) mod textcontrol;
pub(crate) mod textdecoder;
pub(crate) mod textdecodercommon;
pub(crate) mod textdecoderstream;
pub(crate) mod textencoder;
pub(crate) mod textencoderstream;
pub(crate) mod timeranges;
pub(crate) mod toggleevent;
pub(crate) mod touch;
pub(crate) mod touchevent;
pub(crate) mod touchlist;
pub(crate) mod transitionevent;
pub(crate) mod treewalker;
pub(crate) mod trustedhtml;
pub(crate) mod trustedscript;
pub(crate) mod trustedscripturl;
pub(crate) mod trustedtypepolicy;
pub(crate) mod trustedtypepolicyfactory;
pub(crate) mod uievent;
pub(crate) mod url;
pub(crate) mod urlhelper;
pub(crate) mod urlpattern;
pub(crate) mod urlsearchparams;
pub(crate) mod useractivation;
pub(crate) mod userscripts;
pub(crate) mod validation;
pub(crate) mod validitystate;
pub(crate) mod values;
pub(crate) mod virtualmethods;
pub(crate) mod visibilitystateentry;
pub(crate) mod visualviewport;
pub(crate) mod webgl;
pub(crate) use self::webgl::extensions::ext::*;
pub(crate) use self::webgl::*;
pub(crate) mod websocket;
#[cfg(feature = "webxr")]
mod webxr;
#[cfg(feature = "webxr")]
pub(crate) use self::webxr::*;
#[cfg(feature = "webgpu")]
pub(crate) mod webgpu;
#[cfg(feature = "webgpu")]
pub(crate) use self::webgpu::*;
#[cfg(not(feature = "webgpu"))]
pub(crate) mod gpucanvascontext;
pub(crate) mod webrtc;
pub(crate) use self::webrtc::*;
pub(crate) mod webvtt;
pub(crate) use self::webvtt::*;
pub(crate) mod wheelevent;
#[expect(dead_code)]
pub(crate) mod window;
pub(crate) mod windowproxy;
pub(crate) mod workers;
pub(crate) use self::workers::*;
pub(crate) mod worklet;
pub(crate) mod workletglobalscope;
pub(crate) mod xmldocument;
pub(crate) mod xmlhttprequest;
pub(crate) mod xmlhttprequesteventtarget;
pub(crate) mod xmlhttprequestupload;
pub(crate) mod xmlserializer;
pub(crate) mod xpathevaluator;
pub(crate) mod xpathexpression;
pub(crate) mod xpathresult;
