/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_id = "github.com/mozilla/servo#script:0.1"]
#![crate_type = "lib"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(globs, macro_rules, struct_variant, phase)]

#![feature(phase)]

#![allow(non_snake_case_functions)]

#[phase(plugin, link)]
extern crate log;

extern crate debug;
extern crate cssparser;
extern crate collections;
extern crate geom;
extern crate hubbub;
extern crate encoding;
extern crate http;
extern crate js;
extern crate libc;
extern crate native;
extern crate net;
extern crate serialize;
extern crate time;
#[phase(plugin)]
extern crate servo_macros = "macros";
extern crate servo_net = "net";
extern crate servo_util = "util";
extern crate style;
extern crate sync;
extern crate servo_msg = "msg";
extern crate url;

pub mod dom {
    pub mod bindings {
        pub mod global;
        pub mod js;
        pub mod utils;
        pub mod callback;
        pub mod error;
        pub mod conversions;
        pub mod proxyhandler;
        pub mod str;
        pub mod trace;
        pub mod codegen {
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

    pub mod attr;
    pub mod attrlist;
    pub mod blob;
    pub mod browsercontext;
    pub mod characterdata;
    pub mod clientrect;
    pub mod clientrectlist;
    pub mod comment;
    pub mod console;
    pub mod customevent;
    pub mod document;
    pub mod documentfragment;
    pub mod documenttype;
    pub mod domexception;
    pub mod domimplementation;
    pub mod domparser;
    pub mod domtokenlist;
    pub mod element;
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
    pub mod location;
    pub mod mouseevent;
    pub mod navigator;
    pub mod node;
    pub mod nodelist;
    pub mod processinginstruction;
    pub mod performance;
    pub mod performancetiming;
    pub mod progressevent;
    pub mod text;
    pub mod uievent;
    pub mod urlsearchparams;
    pub mod validitystate;
    pub mod virtualmethods;
    pub mod window;
    pub mod xmlhttprequest;
    pub mod xmlhttprequesteventtarget;
    pub mod xmlhttprequestupload;

    pub mod testbinding;
}

pub mod html {
    pub mod cssparse;
    pub mod hubbub_html_parser;
}

pub mod layout_interface;
pub mod page;
pub mod script_task;
