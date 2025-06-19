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
//! [`LayoutDom`](bindings/root/struct.LayoutDom.html) smart pointer. This does not
//! keep the DOM object alive; we ensure that no DOM code (Garbage Collection
//! in particular) runs while layout is accessing the DOM.
//!
//! Methods accessible to layout are implemented on `LayoutDom<Foo>` using
//! `LayoutFooHelpers` traits.

#[macro_use]
pub(crate) mod macros;

#[allow(unused_imports)]
pub(crate) mod types {
    include!(concat!(env!("OUT_DIR"), "/InterfaceTypes.rs"));
}

pub(crate) mod abortcontroller;
pub(crate) mod abortsignal;
#[allow(dead_code)]
pub(crate) mod abstractrange;
pub(crate) mod abstractworker;
pub(crate) mod abstractworkerglobalscope;
pub(crate) mod activation;
pub(crate) mod analysernode;
pub(crate) mod animationevent;
pub(crate) mod attr;
pub(crate) mod audiobuffer;
pub(crate) mod audiobuffersourcenode;
pub(crate) mod audiocontext;
pub(crate) mod audiodestinationnode;
pub(crate) mod audiolistener;
pub(crate) mod audionode;
pub(crate) mod audioparam;
pub(crate) mod audioscheduledsourcenode;
pub(crate) mod audiotrack;
pub(crate) mod audiotracklist;
pub(crate) mod baseaudiocontext;
pub(crate) mod beforeunloadevent;
pub(crate) mod bindings;
pub(crate) mod biquadfilternode;
pub(crate) mod blob;
#[cfg(feature = "bluetooth")]
#[allow(clippy::module_inception)]
pub(crate) mod bluetooth;
#[cfg(feature = "bluetooth")]
pub(crate) use self::bluetooth::*;
pub(crate) mod broadcastchannel;
pub(crate) mod bytelengthqueuingstrategy;
pub(crate) mod canvasgradient;
pub(crate) mod canvaspattern;
#[allow(dead_code)]
pub(crate) mod canvasrenderingcontext2d;
pub(crate) mod cdatasection;
pub(crate) mod channelmergernode;
pub(crate) mod channelsplitternode;
pub(crate) mod characterdata;
pub(crate) mod client;
pub(crate) mod clipboard;
pub(crate) mod clipboardevent;
pub(crate) mod clipboarditem;
pub(crate) mod closeevent;
pub(crate) mod comment;
pub(crate) mod compositionevent;
pub(crate) mod console;
pub(crate) mod constantsourcenode;
pub(crate) mod countqueuingstrategy;
mod create;
pub(crate) mod crypto;
pub(crate) mod cryptokey;
pub(crate) mod csppolicyviolationreport;
pub(crate) mod css;
pub(crate) mod cssconditionrule;
pub(crate) mod cssfontfacerule;
pub(crate) mod cssgroupingrule;
pub(crate) mod cssimportrule;
pub(crate) mod csskeyframerule;
pub(crate) mod csskeyframesrule;
pub(crate) mod csslayerblockrule;
pub(crate) mod csslayerstatementrule;
pub(crate) mod cssmediarule;
pub(crate) mod cssnamespacerule;
pub(crate) mod cssnesteddeclarations;
pub(crate) mod cssrule;
pub(crate) mod cssrulelist;
pub(crate) mod cssstyledeclaration;
pub(crate) mod cssstylerule;
pub(crate) mod cssstylesheet;
pub(crate) mod cssstylevalue;
pub(crate) mod csssupportsrule;
pub(crate) mod customelementregistry;
pub(crate) mod customevent;
pub(crate) mod datatransfer;
pub(crate) mod datatransferitem;
pub(crate) mod datatransferitemlist;
pub(crate) mod dedicatedworkerglobalscope;
pub(crate) mod defaultteereadrequest;
pub(crate) mod defaultteeunderlyingsource;
pub(crate) mod dissimilaroriginlocation;
pub(crate) mod dissimilaroriginwindow;
#[allow(dead_code)]
pub(crate) mod document;
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
pub(crate) mod dynamicmoduleowner;
#[allow(dead_code)]
pub(crate) mod element;
pub(crate) mod elementinternals;
pub(crate) mod errorevent;
pub(crate) mod event;
pub(crate) mod eventsource;
pub(crate) mod eventtarget;
pub(crate) mod extendableevent;
pub(crate) mod extendablemessageevent;
pub(crate) mod file;
pub(crate) mod filelist;
pub(crate) mod filereader;
pub(crate) mod filereadersync;
pub(crate) mod focusevent;
pub(crate) mod fontface;
pub(crate) mod fontfaceset;
pub(crate) mod formdata;
pub(crate) mod formdataevent;
pub(crate) mod gainnode;
pub(crate) mod gamepad;
pub(crate) mod gamepadbutton;
pub(crate) mod gamepadbuttonlist;
pub(crate) mod gamepadevent;
pub(crate) mod gamepadhapticactuator;
pub(crate) mod gamepadpose;
#[allow(dead_code)]
pub(crate) mod globalscope;
pub(crate) mod hashchangeevent;
pub(crate) mod headers;
pub(crate) mod history;
pub(crate) mod htmlanchorelement;
pub(crate) mod htmlareaelement;
pub(crate) mod htmlaudioelement;
pub(crate) mod htmlbaseelement;
pub(crate) mod htmlbodyelement;
pub(crate) mod htmlbrelement;
pub(crate) mod htmlbuttonelement;
#[allow(dead_code)]
pub(crate) mod htmlcanvaselement;
pub(crate) mod htmlcollection;
pub(crate) mod htmldataelement;
pub(crate) mod htmldatalistelement;
pub(crate) mod htmldetailselement;
pub(crate) mod htmldialogelement;
pub(crate) mod htmldirectoryelement;
pub(crate) mod htmldivelement;
pub(crate) mod htmldlistelement;
pub(crate) mod htmlelement;
pub(crate) mod htmlembedelement;
pub(crate) mod htmlfieldsetelement;
pub(crate) mod htmlfontelement;
pub(crate) mod htmlformcontrolscollection;
pub(crate) mod htmlformelement;
pub(crate) mod htmlframeelement;
pub(crate) mod htmlframesetelement;
pub(crate) mod htmlheadelement;
pub(crate) mod htmlheadingelement;
pub(crate) mod htmlhrelement;
pub(crate) mod htmlhtmlelement;
pub(crate) mod htmlhyperlinkelementutils;
pub(crate) mod htmliframeelement;
pub(crate) mod htmlimageelement;
pub(crate) mod htmlinputelement;
pub(crate) mod htmllabelelement;
pub(crate) mod htmllegendelement;
pub(crate) mod htmllielement;
pub(crate) mod htmllinkelement;
pub(crate) mod htmlmapelement;
pub(crate) mod htmlmediaelement;
pub(crate) mod htmlmenuelement;
pub(crate) mod htmlmetaelement;
pub(crate) mod htmlmeterelement;
pub(crate) mod htmlmodelement;
pub(crate) mod htmlobjectelement;
pub(crate) mod htmlolistelement;
pub(crate) mod htmloptgroupelement;
pub(crate) mod htmloptionelement;
pub(crate) mod htmloptionscollection;
pub(crate) mod htmloutputelement;
pub(crate) mod htmlparagraphelement;
pub(crate) mod htmlparamelement;
pub(crate) mod htmlpictureelement;
pub(crate) mod htmlpreelement;
pub(crate) mod htmlprogresselement;
pub(crate) mod htmlquoteelement;
#[allow(dead_code)]
pub(crate) mod htmlscriptelement;
pub(crate) mod htmlselectelement;
pub(crate) mod htmlslotelement;
pub(crate) mod htmlsourceelement;
pub(crate) mod htmlspanelement;
pub(crate) mod htmlstyleelement;
pub(crate) mod htmltablecaptionelement;
pub(crate) mod htmltablecellelement;
pub(crate) mod htmltablecolelement;
pub(crate) mod htmltableelement;
pub(crate) mod htmltablerowelement;
pub(crate) mod htmltablesectionelement;
pub(crate) mod htmltemplateelement;
pub(crate) mod htmltextareaelement;
pub(crate) mod htmltimeelement;
pub(crate) mod htmltitleelement;
pub(crate) mod htmltrackelement;
pub(crate) mod htmlulistelement;
pub(crate) mod htmlunknownelement;
pub(crate) mod htmlvideoelement;
pub(crate) mod idbdatabase;
pub(crate) mod idbfactory;
pub(crate) mod idbobjectstore;
pub(crate) mod idbopendbrequest;
pub(crate) mod idbrequest;
pub(crate) mod idbtransaction;
pub(crate) mod idbversionchangeevent;
pub(crate) mod iirfilternode;
pub(crate) mod imagebitmap;
pub(crate) mod imagedata;
pub(crate) mod inputevent;
pub(crate) mod intersectionobserver;
pub(crate) mod intersectionobserverentry;
pub(crate) mod intersectionobserverrootmargin;
pub(crate) mod keyboardevent;
pub(crate) mod location;
pub(crate) mod mediadeviceinfo;
pub(crate) mod mediadevices;
pub(crate) mod mediaelementaudiosourcenode;
pub(crate) mod mediaerror;
pub(crate) mod mediafragmentparser;
pub(crate) mod medialist;
pub(crate) mod mediametadata;
pub(crate) mod mediaquerylist;
pub(crate) mod mediaquerylistevent;
pub(crate) mod mediasession;
pub(crate) mod mediastream;
pub(crate) mod mediastreamaudiodestinationnode;
pub(crate) mod mediastreamaudiosourcenode;
pub(crate) mod mediastreamtrack;
pub(crate) mod mediastreamtrackaudiosourcenode;
pub(crate) mod messagechannel;
pub(crate) mod messageevent;
#[allow(dead_code)]
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
#[allow(dead_code)]
pub(crate) mod node;
pub(crate) mod nodeiterator;
#[allow(dead_code)]
pub(crate) mod nodelist;
pub(crate) mod notification;
pub(crate) mod offlineaudiocompletionevent;
pub(crate) mod offlineaudiocontext;
pub(crate) mod offscreencanvas;
pub(crate) mod offscreencanvasrenderingcontext2d;
pub(crate) mod oscillatornode;
pub(crate) mod pagetransitionevent;
pub(crate) mod paintrenderingcontext2d;
pub(crate) mod paintsize;
pub(crate) mod paintworkletglobalscope;
pub(crate) mod pannernode;
pub(crate) mod path2d;
pub(crate) mod performance;
#[allow(dead_code)]
pub(crate) mod performanceentry;
pub(crate) mod performancemark;
pub(crate) mod performancemeasure;
pub(crate) mod performancenavigation;
pub(crate) mod performancenavigationtiming;
#[allow(dead_code)]
pub(crate) mod performanceobserver;
pub(crate) mod performanceobserverentrylist;
pub(crate) mod performancepainttiming;
pub(crate) mod performanceresourcetiming;
pub(crate) mod permissions;
pub(crate) mod permissionstatus;
pub(crate) mod plugin;
pub(crate) mod pluginarray;
#[allow(dead_code)]
pub(crate) mod pointerevent;
pub(crate) mod popstateevent;
pub(crate) mod processinginstruction;
pub(crate) mod progressevent;
#[allow(dead_code)]
pub(crate) mod promise;
pub(crate) mod promisenativehandler;
pub(crate) mod promiserejectionevent;
pub(crate) mod radionodelist;
pub(crate) mod range;
pub(crate) mod raredata;
#[allow(dead_code)]
pub(crate) mod readablebytestreamcontroller;
pub(crate) mod readablestream;
pub(crate) mod readablestreambyobreader;
pub(crate) mod readablestreambyobrequest;
pub(crate) mod readablestreamdefaultcontroller;
pub(crate) mod readablestreamdefaultreader;
pub(crate) mod readablestreamgenericreader;
pub(crate) mod request;
pub(crate) mod resizeobserver;
pub(crate) mod resizeobserverentry;
pub(crate) mod resizeobserversize;
pub(crate) mod response;
pub(crate) mod rtcdatachannel;
pub(crate) mod rtcdatachannelevent;
pub(crate) mod rtcerror;
pub(crate) mod rtcerrorevent;
pub(crate) mod rtcicecandidate;
pub(crate) mod rtcpeerconnection;
pub(crate) mod rtcpeerconnectioniceevent;
pub(crate) mod rtcrtpsender;
pub(crate) mod rtcrtptransceiver;
pub(crate) mod rtcsessiondescription;
pub(crate) mod rtctrackevent;
pub(crate) mod screen;
pub(crate) mod securitypolicyviolationevent;
pub(crate) mod selection;
#[allow(dead_code)]
pub(crate) mod serviceworker;
pub(crate) mod serviceworkercontainer;
pub(crate) mod serviceworkerglobalscope;
#[allow(dead_code)]
pub(crate) mod serviceworkerregistration;
pub(crate) mod servointernals;
#[allow(dead_code)]
pub(crate) mod servoparser;
pub(crate) mod shadowroot;
pub(crate) mod staticrange;
pub(crate) mod stereopannernode;
pub(crate) mod storage;
pub(crate) mod storageevent;
pub(crate) mod stylepropertymapreadonly;
pub(crate) mod stylesheet;
pub(crate) mod stylesheetlist;
pub(crate) mod submitevent;
pub(crate) mod subtlecrypto;
pub(crate) mod svgelement;
pub(crate) mod svggraphicselement;
pub(crate) mod svgimageelement;
pub(crate) mod svgsvgelement;
#[cfg(feature = "testbinding")]
pub(crate) mod testbinding;
#[cfg(feature = "testbinding")]
pub(crate) mod testbindingiterable;
#[cfg(feature = "testbinding")]
pub(crate) mod testbindingmaplikewithinterface;
#[cfg(feature = "testbinding")]
pub(crate) mod testbindingmaplikewithprimitive;
#[cfg(feature = "testbinding")]
pub(crate) mod testbindingpairiterable;
#[cfg(feature = "testbinding")]
pub(crate) mod testbindingproxy;
#[cfg(feature = "testbinding")]
pub(crate) mod testbindingsetlikewithinterface;
#[cfg(feature = "testbinding")]
pub(crate) mod testbindingsetlikewithprimitive;
#[cfg(feature = "testbinding")]
pub(crate) mod testns;
#[cfg(feature = "testbinding")]
pub(crate) mod testutils;
#[cfg(feature = "testbinding")]
pub(crate) mod testworklet;
#[cfg(feature = "testbinding")]
pub(crate) mod testworkletglobalscope;
pub(crate) mod text;
pub(crate) mod textcontrol;
pub(crate) mod textdecoder;
pub(crate) mod textencoder;
pub(crate) mod textmetrics;
pub(crate) mod texttrack;
pub(crate) mod texttrackcue;
pub(crate) mod texttrackcuelist;
pub(crate) mod texttracklist;
#[allow(dead_code)]
pub(crate) mod timeranges;
pub(crate) mod touch;
pub(crate) mod touchevent;
pub(crate) mod touchlist;
pub(crate) mod trackevent;
pub(crate) mod transitionevent;
pub(crate) mod treewalker;
pub(crate) mod trustedhtml;
pub(crate) mod trustedscript;
pub(crate) mod trustedscripturl;
pub(crate) mod trustedtypepolicy;
pub(crate) mod trustedtypepolicyfactory;
pub(crate) mod uievent;
pub(crate) mod underlyingsourcecontainer;
pub(crate) mod url;
pub(crate) mod urlhelper;
pub(crate) mod urlpattern;
pub(crate) mod urlsearchparams;
pub(crate) mod userscripts;
pub(crate) mod validation;
pub(crate) mod validitystate;
pub(crate) mod values;
pub(crate) mod vertexarrayobject;
pub(crate) mod videotrack;
pub(crate) mod videotracklist;
pub(crate) mod virtualmethods;
pub(crate) mod visibilitystateentry;
pub(crate) mod vttcue;
pub(crate) mod vttregion;
pub(crate) mod webgl2renderingcontext;
pub(crate) mod webgl_extensions;
pub(crate) mod webgl_validations;
pub(crate) mod webglactiveinfo;
pub(crate) mod webglbuffer;
pub(crate) mod webglcontextevent;
pub(crate) mod webglframebuffer;
pub(crate) mod webglobject;
pub(crate) mod webglprogram;
pub(crate) mod webglquery;
pub(crate) mod webglrenderbuffer;
pub(crate) mod webglrenderingcontext;
pub(crate) mod webglsampler;
pub(crate) mod webglshader;
pub(crate) mod webglshaderprecisionformat;
pub(crate) mod webglsync;
pub(crate) mod webgltexture;
pub(crate) mod webgltransformfeedback;
pub(crate) mod webgluniformlocation;
pub(crate) mod webglvertexarrayobject;
pub(crate) mod webglvertexarrayobjectoes;
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
pub(crate) mod transformstream;
pub(crate) mod transformstreamdefaultcontroller;
pub(crate) mod wheelevent;
#[allow(dead_code)]
pub(crate) mod window;
#[allow(dead_code)]
pub(crate) mod windowproxy;
pub(crate) mod worker;
#[allow(dead_code)]
pub(crate) mod workerglobalscope;
pub(crate) mod workerlocation;
pub(crate) mod workernavigator;
pub(crate) mod worklet;
pub(crate) mod workletglobalscope;
pub(crate) mod writablestream;
pub(crate) mod writablestreamdefaultcontroller;
pub(crate) mod writablestreamdefaultwriter;
pub(crate) mod xmldocument;
pub(crate) mod xmlhttprequest;
pub(crate) mod xmlhttprequesteventtarget;
pub(crate) mod xmlhttprequestupload;
pub(crate) mod xmlserializer;
pub(crate) mod xpathevaluator;
pub(crate) mod xpathexpression;
pub(crate) mod xpathresult;
pub(crate) use self::webgl_extensions::ext::*;
