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
//! * rooting pointers on the stack: the [`Root`](bindings/js/struct.Root.html)
//!   and [`JSRef`](bindings/js/struct.JSRef.html) smart pointers;
//! * tracing pointers in member fields: the [`JS`](bindings/js/struct.JS.html),
//!   [`MutNullableJS`](bindings/js/struct.MutNullableJS.html) and
//!   [`MutHeap`](bindings/js/struct.MutHeap.html) smart pointers and
//!   [the tracing implementation](bindings/trace/index.html);
//! * returning pointers from functions: the
//!   [`Temporary`](bindings/js/struct.Temporary.html) smart pointer;
//! * rooting pointers from across task boundaries or in channels: the
//!   [`Trusted`](bindings/refcounted/struct.Trusted.html) smart pointer;
//! * extracting pointers to DOM objects from their reflectors: the
//!   [`Unrooted`](bindings/js/struct.Unrooted.html) smart pointer.
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
//! Construction
//! ============
//!
//! DOM objects of type `T` in Servo have two constructors:
//!
//! * a `T::new_inherited` static method that returns a plain `T`, and
//! * a `T::new` static method that returns `Temporary<T>`.
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
//! `Reflector` and `Reflectable`
//! =============================
//!
//! Every DOM object has a `Reflector` as its first (transitive) member field.
//! This contains a `*mut JSObject` that points to its reflector.
//!
//! The `FooBinding::Wrap` function creates the reflector, stores a pointer to
//! the DOM object in the reflector, and initializes the pointer to the reflector
//! in the `Reflector` field.
//!
//! The `Reflectable` trait provides a `reflector()` method that returns the
//! DOM object's `Reflector`. It is implemented automatically for DOM structs
//! through the `#[dom_struct]` attribute.
//!
//! Implementing methods for a DOM object
//! =====================================
//!
//! In order to ensure that DOM objects are rooted when they are called, we
//! require that all methods are implemented for `JSRef<'a, Foo>`. This means
//! that all methods are defined on traits. Conventionally, those traits are
//! called
//!
//! * `dom::bindings::codegen::Bindings::FooBindings::FooMethods` for methods
//!   defined through IDL;
//! * `FooHelpers` for public methods;
//! * `PrivateFooHelpers` for private methods.
//!
//! Calling methods on a DOM object
//! ===============================
//!
//! To call a method on a DOM object, we require that the object is rooted, by
//! calling `.root()` on a `Temporary` or `JS` pointer. This constructs a
//! `Root` on the stack, which ensures the DOM object stays alive for the
//! duration of its lifetime. A `JSRef` on which to call the method can then be
//! obtained by calling the `r()` method on the `Root`.
//!
//! Accessing fields of a DOM object
//! ================================
//!
//! All fields of DOM objects are private; accessing them from outside their
//! module is done through explicit getter or setter methods.
//!
//! However, `JSRef<T>` dereferences to `&T`, so fields can be accessed on a
//! `JSRef<T>` directly within the module that defines the struct.
//!
//! Inheritance and casting
//! =======================
//!
//! For all DOM interfaces `Foo` in an inheritance chain, a
//! `dom::bindings::codegen::InheritTypes::FooCast` provides methods to cast
//! to other types in the inheritance chain. For example:
//!
//! ```ignore
//! # use script::dom::bindings::js::JSRef;
//! # use script::dom::bindings::codegen::InheritTypes::{NodeCast, HTMLElementCast};
//! # use script::dom::element::Element;
//! # use script::dom::node::Node;
//! # use script::dom::htmlelement::HTMLElement;
//! fn f(element: JSRef<Element>) {
//!     let base: JSRef<Node> = NodeCast::from_ref(element);
//!     let derived: Option<JSRef<HTMLElement>> = HTMLElementCast::to_ref(element);
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
//! * listing `foo.rs` in components/script/dom/mod.rs`;
//! * defining the DOM struct `Foo` with a `#[dom_struct]` attribute, a
//!   superclass or `Reflector` member, and other members as appropriate;
//! * implementing the
//!   `dom::bindings::codegen::Bindings::FooBindings::FooMethods` trait for
//!   `JSRef<Foo>`.
//!
//! More information is available in the [bindings module](bindings/index.html).
//!
//! Accessing DOM objects from layout
//! =================================
//!
//! Layout code can access the DOM through the
//! [`LayoutJS`](bindings/js/struct.LayoutJS.html) smart pointer. This does not
//! keep the DOM object alive; we ensure that no DOM code (Garbage Collection
//! in particular) runs while the layout task is accessing the DOM.
//!
//! Methods accessible to layout are implemented on `LayoutJS<Foo>` using
//! `LayoutFooHelpers` traits.

#[macro_use]
pub mod macros;

pub mod bindings;

#[path="bindings/codegen/InterfaceTypes.rs"]
pub mod types;

pub mod activation;
pub mod attr;
pub mod blob;
pub mod browsercontext;
pub mod canvasgradient;
pub mod canvaspattern;
pub mod canvasrenderingcontext2d;
pub mod characterdata;
pub mod cssstyledeclaration;
pub mod domrect;
pub mod domrectlist;
pub mod domstringmap;
pub mod comment;
pub mod console;
mod create;
pub mod customevent;
pub mod dedicatedworkerglobalscope;
pub mod document;
pub mod documentfragment;
pub mod documenttype;
pub mod domexception;
pub mod domimplementation;
pub mod domparser;
pub mod domtokenlist;
pub mod element;
pub mod errorevent;
pub mod event;
pub mod eventdispatcher;
pub mod eventtarget;
pub mod file;
pub mod formdata;
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
pub mod htmldialogelement;
pub mod htmldirectoryelement;
pub mod htmldivelement;
pub mod htmldlistelement;
pub mod htmlelement;
pub mod htmlembedelement;
pub mod htmlfieldsetelement;
pub mod htmlfontelement;
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
pub mod htmloutputelement;
pub mod htmlparagraphelement;
pub mod htmlparamelement;
pub mod htmlpreelement;
pub mod htmlprogresselement;
pub mod htmlquoteelement;
pub mod htmlscriptelement;
pub mod htmlselectelement;
pub mod htmlspanelement;
pub mod htmlsourceelement;
pub mod htmlstyleelement;
pub mod htmltableelement;
pub mod htmltablecaptionelement;
pub mod htmltablecellelement;
pub mod htmltabledatacellelement;
pub mod htmltableheadercellelement;
pub mod htmltablecolelement;
pub mod htmltablerowelement;
pub mod htmltablesectionelement;
pub mod htmltemplateelement;
pub mod htmltextareaelement;
pub mod htmltimeelement;
pub mod htmltitleelement;
pub mod htmltrackelement;
pub mod htmlulistelement;
pub mod htmlvideoelement;
pub mod htmlunknownelement;
pub mod imagedata;
pub mod keyboardevent;
pub mod location;
pub mod messageevent;
pub mod mouseevent;
pub mod namednodemap;
pub mod navigator;
pub mod navigatorinfo;
pub mod node;
pub mod nodeiterator;
pub mod nodelist;
pub mod processinginstruction;
pub mod performance;
pub mod performancetiming;
pub mod progressevent;
pub mod range;
pub mod screen;
pub mod servohtmlparser;
pub mod storage;
pub mod storageevent;
pub mod text;
pub mod textdecoder;
pub mod textencoder;
pub mod treewalker;
pub mod uievent;
pub mod urlhelper;
pub mod urlsearchparams;
pub mod userscripts;
pub mod validitystate;
pub mod virtualmethods;
pub mod webglrenderingcontext;
pub mod websocket;
pub mod window;
pub mod worker;
pub mod workerglobalscope;
pub mod workerlocation;
pub mod workernavigator;
pub mod xmlhttprequest;
pub mod xmlhttprequesteventtarget;
pub mod xmlhttprequestupload;

pub mod testbinding;
