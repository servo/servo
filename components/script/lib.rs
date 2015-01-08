/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(default_type_params, globs, macro_rules, phase, unsafe_destructor, if_let)]

#![deny(unused_imports)]
#![deny(unused_variables)]
#![allow(non_snake_case)]

#![doc="The script crate contains all matters DOM."]

#[phase(plugin, link)]
extern crate log;

extern crate devtools_traits;
extern crate cssparser;
extern crate collections;
extern crate geom;
extern crate html5ever;
extern crate encoding;
extern crate hyper;
extern crate js;
extern crate libc;
extern crate msg;
extern crate net;
extern crate rustrt;
extern crate serialize;
extern crate time;
extern crate canvas;
extern crate script_traits;
#[phase(plugin)]
extern crate "plugins" as servo_plugins;
extern crate "net" as servo_net;
extern crate "util" as servo_util;
extern crate style;
extern crate "msg" as servo_msg;
extern crate url;
extern crate uuid;
extern crate string_cache;
#[phase(plugin)]
extern crate string_cache_macros;

pub mod cors;

/// The implementation of the DOM.
#[macro_escape]
pub mod dom {
    #[macro_escape]
    pub mod macros;

    /// The code to expose the DOM to JavaScript through IDL bindings.
    pub mod bindings {
        pub mod cell;
        pub mod global;
        pub mod js;
        pub mod refcounted;
        pub mod utils;
        pub mod callback;
        pub mod error;
        pub mod conversions;
        mod proxyhandler;
        pub mod str;
        pub mod trace;

        /// Generated JS-Rust bindings.
        pub mod codegen {
            #[allow(unrooted_must_root)]
            pub mod Bindings;
            pub mod InterfaceTypes;
            pub mod InheritTypes;
            pub mod PrototypeList;
            pub mod RegisterBindings;
            pub mod UnionTypes;
        }
    }

    #[path="bindings/codegen/InterfaceTypes.rs"]
    pub mod types;

    pub mod activation;
    pub mod attr;
    pub mod blob;
    pub mod browsercontext;
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
    pub mod htmlserializer;
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
    pub mod text;
    pub mod treewalker;
    pub mod uievent;
    pub mod urlhelper;
    pub mod urlsearchparams;
    pub mod validitystate;
    pub mod virtualmethods;
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
}

pub mod parse;

pub mod layout_interface;
pub mod page;
pub mod script_task;
mod timers;
pub mod textinput;
mod devtools;

#[cfg(all(test, target_word_size = "64"))]
mod tests;
