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
#[phase(syntax, link)]
extern crate log;

extern crate collections;
extern crate geom;
extern crate hubbub;
extern crate encoding;
extern crate js;
extern crate libc;
extern crate native;
extern crate serialize;
extern crate time;
#[phase(syntax)]
extern crate servo_macros = "macros";
extern crate servo_net = "net";
extern crate servo_util = "util";
extern crate style;
extern crate servo_msg = "msg";
extern crate url;

pub mod dom {
    pub mod bindings {
        pub mod js;
        pub mod element;
        pub mod utils;
        pub mod callback;
        pub mod error;
        pub mod conversions;
        pub mod proxyhandler;
        pub mod str;
        pub mod trace;
        pub mod codegen {
            pub use self::BindingDeclarations::{AttrBinding,
                                                AttrListBinding,
                                                BlobBinding,
                                                CharacterDataBinding,
                                                ClientRectBinding,
                                                ClientRectListBinding,
                                                CommentBinding,
                                                ConsoleBinding,
                                                DOMExceptionBinding,
                                                DOMImplementationBinding,
                                                DOMParserBinding,
                                                DocumentBinding,
                                                DocumentFragmentBinding,
                                                DocumentTypeBinding,
                                                ElementBinding,
                                                EventBinding,
                                                EventListenerBinding,
                                                EventTargetBinding,
                                                FormDataBinding,
                                                HTMLAnchorElementBinding,
                                                HTMLAppletElementBinding,
                                                HTMLAreaElementBinding,
                                                HTMLAudioElementBinding,
                                                HTMLBRElementBinding,
                                                HTMLBaseElementBinding,
                                                HTMLBodyElementBinding,
                                                HTMLButtonElementBinding,
                                                HTMLCanvasElementBinding,
                                                HTMLCollectionBinding,
                                                HTMLDListElementBinding,
                                                HTMLDataElementBinding,
                                                HTMLDataListElementBinding,
                                                HTMLDirectoryElementBinding,
                                                HTMLDivElementBinding,
                                                HTMLElementBinding,
                                                HTMLEmbedElementBinding,
                                                HTMLFieldSetElementBinding,
                                                HTMLFontElementBinding,
                                                HTMLFormElementBinding,
                                                HTMLFrameElementBinding,
                                                HTMLFrameSetElementBinding,
                                                HTMLHRElementBinding,
                                                HTMLHeadElementBinding,
                                                HTMLHeadingElementBinding,
                                                HTMLHtmlElementBinding,
                                                HTMLIFrameElementBinding,
                                                HTMLImageElementBinding,
                                                HTMLInputElementBinding,
                                                HTMLLIElementBinding,
                                                HTMLLabelElementBinding,
                                                HTMLLegendElementBinding,
                                                HTMLLinkElementBinding,
                                                HTMLMainElementBinding,
                                                HTMLMapElementBinding,
                                                HTMLMediaElementBinding,
                                                HTMLMetaElementBinding,
                                                HTMLMeterElementBinding,
                                                HTMLModElementBinding,
                                                HTMLOListElementBinding,
                                                HTMLObjectElementBinding,
                                                HTMLOptGroupElementBinding,
                                                HTMLOptionElementBinding,
                                                HTMLOutputElementBinding,
                                                HTMLParagraphElementBinding,
                                                HTMLParamElementBinding,
                                                HTMLPreElementBinding,
                                                HTMLProgressElementBinding,
                                                HTMLQuoteElementBinding,
                                                HTMLScriptElementBinding,
                                                HTMLSelectElementBinding,
                                                HTMLSourceElementBinding,
                                                HTMLSpanElementBinding,
                                                HTMLStyleElementBinding,
                                                HTMLTableCaptionElementBinding,
                                                HTMLTableCellElementBinding,
                                                HTMLTableColElementBinding,
                                                HTMLTableDataCellElementBinding,
                                                HTMLTableElementBinding,
                                                HTMLTableHeaderCellElementBinding,
                                                HTMLTableRowElementBinding,
                                                HTMLTableSectionElementBinding,
                                                HTMLTemplateElementBinding,
                                                HTMLTextAreaElementBinding,
                                                HTMLTimeElementBinding,
                                                HTMLTitleElementBinding,
                                                HTMLTrackElementBinding,
                                                HTMLUListElementBinding,
                                                HTMLUnknownElementBinding,
                                                HTMLVideoElementBinding,
                                                LocationBinding,
                                                MouseEventBinding,
                                                NavigatorBinding,
                                                NodeBinding,
                                                NodeListBinding,
                                                ProcessingInstructionBinding,
                                                TestBindingBinding,
                                                TextBinding,
                                                UIEventBinding,
                                                ValidityStateBinding,
                                                WindowBinding};
            pub mod InheritTypes;
            pub mod PrototypeList;
            pub mod RegisterBindings;
            pub mod BindingDeclarations;
            pub mod UnionTypes;
        }
    }

    pub mod types {
        pub use super::attr::Attr;
        pub use super::attrlist::AttrList;
        pub use super::blob::Blob;
        pub use super::characterdata::CharacterData;
        pub use super::clientrect::ClientRect;
        pub use super::clientrectlist::ClientRectList;
        pub use super::comment::Comment;
        pub use super::console::Console;
        pub use super::domexception::DOMException;
        pub use super::domimplementation::DOMImplementation;
        pub use super::domparser::DOMParser;
        pub use super::document::Document;
        pub use super::documentfragment::DocumentFragment;
        pub use super::documenttype::DocumentType;
        pub use super::element::Element;
        pub use super::event::Event;
        pub use super::eventtarget::EventTarget;
        pub use super::formdata::FormData;
        pub use super::htmlanchorelement::HTMLAnchorElement;
        pub use super::htmlappletelement::HTMLAppletElement;
        pub use super::htmlareaelement::HTMLAreaElement;
        pub use super::htmlaudioelement::HTMLAudioElement;
        pub use super::htmlbrelement::HTMLBRElement;
        pub use super::htmlbaseelement::HTMLBaseElement;
        pub use super::htmlbodyelement::HTMLBodyElement;
        pub use super::htmlbuttonelement::HTMLButtonElement;
        pub use super::htmlcanvaselement::HTMLCanvasElement;
        pub use super::htmlcollection::HTMLCollection;
        pub use super::htmldlistelement::HTMLDListElement;
        pub use super::htmldataelement::HTMLDataElement;
        pub use super::htmldatalistelement::HTMLDataListElement;
        pub use super::htmldirectoryelement::HTMLDirectoryElement;
        pub use super::htmldivelement::HTMLDivElement;
        pub use super::htmlelement::HTMLElement;
        pub use super::htmlembedelement::HTMLEmbedElement;
        pub use super::htmlfieldsetelement::HTMLFieldSetElement;
        pub use super::htmlfontelement::HTMLFontElement;
        pub use super::htmlformelement::HTMLFormElement;
        pub use super::htmlframeelement::HTMLFrameElement;
        pub use super::htmlframesetelement::HTMLFrameSetElement;
        pub use super::htmlhrelement::HTMLHRElement;
        pub use super::htmlheadelement::HTMLHeadElement;
        pub use super::htmlheadingelement::HTMLHeadingElement;
        pub use super::htmlhtmlelement::HTMLHtmlElement;
        pub use super::htmliframeelement::HTMLIFrameElement;
        pub use super::htmlimageelement::HTMLImageElement;
        pub use super::htmlinputelement::HTMLInputElement;
        pub use super::htmllielement::HTMLLIElement;
        pub use super::htmllabelelement::HTMLLabelElement;
        pub use super::htmllegendelement::HTMLLegendElement;
        pub use super::htmllinkelement::HTMLLinkElement;
        pub use super::htmlmainelement::HTMLMainElement;
        pub use super::htmlmapelement::HTMLMapElement;
        pub use super::htmlmediaelement::HTMLMediaElement;
        pub use super::htmlmetaelement::HTMLMetaElement;
        pub use super::htmlmeterelement::HTMLMeterElement;
        pub use super::htmlmodelement::HTMLModElement;
        pub use super::htmlolistelement::HTMLOListElement;
        pub use super::htmlobjectelement::HTMLObjectElement;
        pub use super::htmloptgroupelement::HTMLOptGroupElement;
        pub use super::htmloptionelement::HTMLOptionElement;
        pub use super::htmloutputelement::HTMLOutputElement;
        pub use super::htmlparagraphelement::HTMLParagraphElement;
        pub use super::htmlparamelement::HTMLParamElement;
        pub use super::htmlpreelement::HTMLPreElement;
        pub use super::htmlprogresselement::HTMLProgressElement;
        pub use super::htmlquoteelement::HTMLQuoteElement;
        pub use super::htmlscriptelement::HTMLScriptElement;
        pub use super::htmlselectelement::HTMLSelectElement;
        pub use super::htmlsourceelement::HTMLSourceElement;
        pub use super::htmlspanelement::HTMLSpanElement;
        pub use super::htmlstyleelement::HTMLStyleElement;
        pub use super::htmltablecaptionelement::HTMLTableCaptionElement;
        pub use super::htmltablecellelement::HTMLTableCellElement;
        pub use super::htmltablecolelement::HTMLTableColElement;
        pub use super::htmltabledatacellelement::HTMLTableDataCellElement;
        pub use super::htmltableelement::HTMLTableElement;
        pub use super::htmltableheadercellelement::HTMLTableHeaderCellElement;
        pub use super::htmltablerowelement::HTMLTableRowElement;
        pub use super::htmltablesectionelement::HTMLTableSectionElement;
        pub use super::htmltemplateelement::HTMLTemplateElement;
        pub use super::htmltextareaelement::HTMLTextAreaElement;
        pub use super::htmltimeelement::HTMLTimeElement;
        pub use super::htmltitleelement::HTMLTitleElement;
        pub use super::htmltrackelement::HTMLTrackElement;
        pub use super::htmlulistelement::HTMLUListElement;
        pub use super::htmlunknownelement::HTMLUnknownElement;
        pub use super::htmlvideoelement::HTMLVideoElement;
        pub use super::location::Location;
        pub use super::mouseevent::MouseEvent;
        pub use super::navigator::Navigator;
        pub use super::node::Node;
        pub use super::nodelist::NodeList;
        pub use super::processinginstruction::ProcessingInstruction;
        pub use super::testbinding::TestBinding;
        pub use super::text::Text;
        pub use super::uievent::UIEvent;
        pub use super::validitystate::ValidityState;
        pub use super::window::Window;
    }

    pub mod attr;
    pub mod attrlist;
    pub mod blob;
    pub mod browsercontext;
    pub mod characterdata;
    pub mod clientrect;
    pub mod clientrectlist;
    pub mod comment;
    pub mod console;
    pub mod document;
    pub mod documentfragment;
    pub mod documenttype;
    pub mod domexception;
    pub mod domimplementation;
    pub mod domparser;
    pub mod element;
    pub mod event;
    pub mod eventdispatcher;
    pub mod eventtarget;
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
    pub mod htmlmainelement;
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
    pub mod uievent;
    pub mod text;
    pub mod validitystate;
    pub mod virtualmethods;
    pub mod window;

    pub mod testbinding;
}

pub mod html {
    pub mod cssparse;
    pub mod hubbub_html_parser;
}

pub mod layout_interface;
pub mod script_task;
