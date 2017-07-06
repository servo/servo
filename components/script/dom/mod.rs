/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
//!   the [`Root`](bindings/js/struct.Root.html) smart pointer;
//! * tracing pointers in member fields: the [`JS`](bindings/js/struct.JS.html),
//!   [`MutNullableJS`](bindings/js/struct.MutNullableJS.html) and
//!   [`MutJS`](bindings/js/struct.MutJS.html) smart pointers and
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
//! it is stored by value, rather than in a smart pointer such as `JS<T>`.)
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
//! * a `T::new` static method that returns `Root<T>`.
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
//! not allowed. In particular, any mutable fields use `Cell` or `DOMRefCell`
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
//! * `dom::bindings::codegen::Bindings::FooBindings::FooMethods` for methods
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
//!   `dom::bindings::codegen::Bindings::FooBindings::FooMethods` trait for
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
//! [`LayoutJS`](bindings/js/struct.LayoutJS.html) smart pointer. This does not
//! keep the DOM object alive; we ensure that no DOM code (Garbage Collection
//! in particular) runs while the layout thread is accessing the DOM.
//!
//! Methods accessible to layout are implemented on `LayoutJS<Foo>` using
//! `LayoutFooHelpers` traits.

#[macro_use]
pub mod macros;

pub mod types {
    #[cfg(not(target_env = "msvc"))]
    include!(concat!(env!("OUT_DIR"), "/InterfaceTypes.rs"));
    #[cfg(target_env = "msvc")]
    include!(concat!(env!("OUT_DIR"), "/build/InterfaceTypes.rs"));
}

pub mod abstractworker;
pub mod abstractworkerglobalscope;
pub mod activation;
pub mod attr;
pub mod beforeunloadevent;
pub mod bindings;
pub mod blob;
pub mod bluetooth;
pub mod bluetoothadvertisingevent;
pub mod bluetoothcharacteristicproperties;
pub mod bluetoothdevice;
pub mod bluetoothpermissionresult;
pub mod bluetoothremotegattcharacteristic;
pub mod bluetoothremotegattdescriptor;
pub mod bluetoothremotegattserver;
pub mod bluetoothremotegattservice;
pub mod bluetoothuuid;
pub mod canvasgradient;
pub mod canvaspattern;
pub mod canvasrenderingcontext2d;
pub mod characterdata;
pub mod client;
pub mod closeevent;
pub mod comment;
pub mod console;
mod create;
pub mod crypto;
pub mod css;
pub mod cssconditionrule;
pub mod cssfontfacerule;
pub mod cssgroupingrule;
pub mod cssimportrule;
pub mod csskeyframerule;
pub mod csskeyframesrule;
pub mod cssmediarule;
pub mod cssnamespacerule;
pub mod cssrule;
pub mod cssrulelist;
pub mod cssstyledeclaration;
pub mod cssstylerule;
pub mod cssstylesheet;
pub mod csssupportsrule;
pub mod cssviewportrule;
pub mod customelementregistry;
pub mod customevent;
pub mod dedicatedworkerglobalscope;
pub mod dissimilaroriginlocation;
pub mod dissimilaroriginwindow;
pub mod document;
pub mod documentfragment;
pub mod documenttype;
pub mod domexception;
pub mod domimplementation;
pub mod dommatrix;
pub mod dommatrixreadonly;
pub mod domparser;
pub mod dompoint;
pub mod dompointreadonly;
pub mod domquad;
pub mod domrect;
pub mod domrectreadonly;
pub mod domstringmap;
pub mod domtokenlist;
pub mod element;
pub mod errorevent;
pub mod event;
pub mod eventsource;
pub mod eventtarget;
pub mod extendableevent;
pub mod extendablemessageevent;
pub mod file;
pub mod filelist;
pub mod filereader;
pub mod filereadersync;
pub mod focusevent;
pub mod forcetouchevent;
pub mod formdata;
pub mod gamepad;
pub mod gamepadbutton;
pub mod gamepadbuttonlist;
pub mod gamepadevent;
pub mod gamepadlist;
pub mod globalscope;
pub mod hashchangeevent;
pub mod headers;
pub mod history;
pub mod htmlanchorelement;
pub mod htmlappletelement;
pub mod htmlareaelement;
pub mod htmlaudioelement;
pub mod htmlbaseelement;
pub mod htmlbodyelement;
pub mod htmlbrelement;
pub mod htmlbuttonelement;
pub mod htmlcanvaselement;
pub mod htmlcollection;
pub mod htmldataelement;
pub mod htmldatalistelement;
pub mod htmldetailselement;
pub mod htmldialogelement;
pub mod htmldirectoryelement;
pub mod htmldivelement;
pub mod htmldlistelement;
pub mod htmlelement;
pub mod htmlembedelement;
pub mod htmlfieldsetelement;
pub mod htmlfontelement;
pub mod htmlformcontrolscollection;
pub mod htmlformelement;
pub mod htmlframeelement;
pub mod htmlframesetelement;
pub mod htmlheadelement;
pub mod htmlheadingelement;
pub mod htmlhrelement;
pub mod htmlhtmlelement;
pub mod htmliframeelement;
pub mod htmlimageelement;
pub mod htmlinputelement;
pub mod htmllabelelement;
pub mod htmllegendelement;
pub mod htmllielement;
pub mod htmllinkelement;
pub mod htmlmapelement;
pub mod htmlmediaelement;
pub mod htmlmetaelement;
pub mod htmlmeterelement;
pub mod htmlmodelement;
pub mod htmlobjectelement;
pub mod htmlolistelement;
pub mod htmloptgroupelement;
pub mod htmloptionelement;
pub mod htmloptionscollection;
pub mod htmloutputelement;
pub mod htmlparagraphelement;
pub mod htmlparamelement;
pub mod htmlpreelement;
pub mod htmlprogresselement;
pub mod htmlquoteelement;
pub mod htmlscriptelement;
pub mod htmlselectelement;
pub mod htmlsourceelement;
pub mod htmlspanelement;
pub mod htmlstyleelement;
pub mod htmltablecaptionelement;
pub mod htmltablecellelement;
pub mod htmltablecolelement;
pub mod htmltabledatacellelement;
pub mod htmltableelement;
pub mod htmltableheadercellelement;
pub mod htmltablerowelement;
pub mod htmltablesectionelement;
pub mod htmltemplateelement;
pub mod htmltextareaelement;
pub mod htmltimeelement;
pub mod htmltitleelement;
pub mod htmltrackelement;
pub mod htmlulistelement;
pub mod htmlunknownelement;
pub mod htmlvideoelement;
pub mod imagedata;
pub mod inputevent;
pub mod keyboardevent;
pub mod location;
pub mod mediaerror;
pub mod medialist;
pub mod mediaquerylist;
pub mod mediaquerylistevent;
pub mod messageevent;
pub mod mimetype;
pub mod mimetypearray;
pub mod mouseevent;
pub mod mutationobserver;
pub mod mutationrecord;
pub mod namednodemap;
pub mod navigator;
pub mod navigatorinfo;
pub mod node;
pub mod nodeiterator;
pub mod nodelist;
pub mod pagetransitionevent;
pub mod paintrenderingcontext2d;
pub mod paintsize;
pub mod paintworkletglobalscope;
pub mod performance;
pub mod performancetiming;
pub mod permissions;
pub mod permissionstatus;
pub mod plugin;
pub mod pluginarray;
pub mod popstateevent;
pub mod processinginstruction;
pub mod progressevent;
pub mod promise;
pub mod promisenativehandler;
pub mod radionodelist;
pub mod range;
pub mod request;
pub mod response;
pub mod screen;
pub mod serviceworker;
pub mod serviceworkercontainer;
pub mod serviceworkerglobalscope;
pub mod serviceworkerregistration;
pub mod servoparser;
pub mod storage;
pub mod storageevent;
pub mod stylesheet;
pub mod stylesheetlist;
pub mod svgelement;
pub mod svggraphicselement;
pub mod svgsvgelement;
pub mod testbinding;
pub mod testbindingiterable;
pub mod testbindingpairiterable;
pub mod testbindingproxy;
pub mod testrunner;
pub mod testworklet;
pub mod testworkletglobalscope;
pub mod text;
pub mod textdecoder;
pub mod textencoder;
pub mod touch;
pub mod touchevent;
pub mod touchlist;
pub mod transitionevent;
pub mod treewalker;
pub mod uievent;
pub mod url;
pub mod urlhelper;
pub mod urlsearchparams;
pub mod userscripts;
pub mod validation;
pub mod validitystate;
pub mod values;
pub mod virtualmethods;
pub mod vr;
pub mod vrdisplay;
pub mod vrdisplaycapabilities;
pub mod vrdisplayevent;
pub mod vreyeparameters;
pub mod vrfieldofview;
pub mod vrframedata;
pub mod vrpose;
pub mod vrstageparameters;
pub mod webgl_extensions;
pub use self::webgl_extensions::ext::*;
pub mod webgl_validations;
pub mod webglactiveinfo;
pub mod webglbuffer;
pub mod webglcontextevent;
pub mod webglframebuffer;
pub mod webglobject;
pub mod webglprogram;
pub mod webglrenderbuffer;
pub mod webglrenderingcontext;
pub mod webglshader;
pub mod webglshaderprecisionformat;
pub mod webgltexture;
pub mod webgluniformlocation;
pub mod websocket;
pub mod window;
pub mod windowproxy;
pub mod worker;
pub mod workerglobalscope;
pub mod workerlocation;
pub mod workernavigator;
pub mod worklet;
pub mod workletglobalscope;
pub mod xmldocument;
pub mod xmlhttprequest;
pub mod xmlhttprequesteventtarget;
pub mod xmlhttprequestupload;
