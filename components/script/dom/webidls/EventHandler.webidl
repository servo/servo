/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://html.spec.whatwg.org/multipage/#eventhandler
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

[TreatNonObjectAsNull]
callback EventHandlerNonNull = any (Event event);
typedef EventHandlerNonNull? EventHandler;

[TreatNonObjectAsNull]
callback OnErrorEventHandlerNonNull = any ((Event or DOMString) event, optional DOMString source,
                                               optional unsigned long lineno, optional unsigned long column,
                                               optional any error);
typedef OnErrorEventHandlerNonNull? OnErrorEventHandler;

[TreatNonObjectAsNull]
callback OnBeforeUnloadEventHandlerNonNull = DOMString? (Event event);
typedef OnBeforeUnloadEventHandlerNonNull? OnBeforeUnloadEventHandler;

// https://html.spec.whatwg.org/multipage/#globaleventhandlers
[NoInterfaceObject, Exposed=Window]
interface GlobalEventHandlers {
           attribute EventHandler onabort;
           attribute EventHandler onblur;
           attribute EventHandler oncancel;
           attribute EventHandler oncanplay;
           attribute EventHandler oncanplaythrough;
           attribute EventHandler onchange;
           attribute EventHandler onclick;
           attribute EventHandler onclose;
           attribute EventHandler oncontextmenu;
           attribute EventHandler oncuechange;
           attribute EventHandler ondblclick;
           attribute EventHandler ondrag;
           attribute EventHandler ondragend;
           attribute EventHandler ondragenter;
           attribute EventHandler ondragexit;
           attribute EventHandler ondragleave;
           attribute EventHandler ondragover;
           attribute EventHandler ondragstart;
           attribute EventHandler ondrop;
           attribute EventHandler ondurationchange;
           attribute EventHandler onemptied;
           attribute EventHandler onended;
           attribute OnErrorEventHandler onerror;
           attribute EventHandler onfocus;
           attribute EventHandler oninput;
           attribute EventHandler oninvalid;
           attribute EventHandler onkeydown;
           attribute EventHandler onkeypress;
           attribute EventHandler onkeyup;
           attribute EventHandler onload;
           attribute EventHandler onloadeddata;
           attribute EventHandler onloadedmetadata;
           attribute EventHandler onloadstart;
           attribute EventHandler onmousedown;
           [LenientThis] attribute EventHandler onmouseenter;
           [LenientThis] attribute EventHandler onmouseleave;
           attribute EventHandler onmousemove;
           attribute EventHandler onmouseout;
           attribute EventHandler onmouseover;
           attribute EventHandler onmouseup;
           attribute EventHandler onwheel;
           attribute EventHandler onpause;
           attribute EventHandler onplay;
           attribute EventHandler onplaying;
           attribute EventHandler onprogress;
           attribute EventHandler onratechange;
           attribute EventHandler onreset;
           attribute EventHandler onresize;
           attribute EventHandler onscroll;
           attribute EventHandler onseeked;
           attribute EventHandler onseeking;
           attribute EventHandler onselect;
           attribute EventHandler onshow;
           attribute EventHandler onstalled;
           attribute EventHandler onsubmit;
           attribute EventHandler onsuspend;
           attribute EventHandler ontimeupdate;
           attribute EventHandler ontoggle;
           attribute EventHandler onvolumechange;
           attribute EventHandler onwaiting;
};

// https://drafts.csswg.org/css-transitions/#interface-globaleventhandlers-idl
partial interface GlobalEventHandlers {
           attribute EventHandler ontransitionend;
};

// https://html.spec.whatwg.org/multipage/#windoweventhandlers
[NoInterfaceObject, Exposed=Window]
interface WindowEventHandlers {
           attribute EventHandler onafterprint;
           attribute EventHandler onbeforeprint;
           attribute OnBeforeUnloadEventHandler onbeforeunload;
           attribute EventHandler onhashchange;
           attribute EventHandler onlanguagechange;
           attribute EventHandler onmessage;
           attribute EventHandler onoffline;
           attribute EventHandler ononline;
           attribute EventHandler onpagehide;
           attribute EventHandler onpageshow;
           attribute EventHandler onpopstate;
           attribute EventHandler onrejectionhandled;
           attribute EventHandler onstorage;
           attribute EventHandler onunhandledrejection;
           attribute EventHandler onunload;
};

// https://w3c.github.io/webvr/spec/1.1/#interface-window
partial interface WindowEventHandlers {
           attribute EventHandler onvrdisplayconnect;
           attribute EventHandler onvrdisplaydisconnect;
           attribute EventHandler onvrdisplayactivate;
           attribute EventHandler onvrdisplaydeactivate;
           attribute EventHandler onvrdisplayblur;
           attribute EventHandler onvrdisplayfocus;
           attribute EventHandler onvrdisplaypresentchange;
};

// https://html.spec.whatwg.org/multipage/#documentandelementeventhandlers
[NoInterfaceObject, Exposed=Window]
interface DocumentAndElementEventHandlers {
          attribute EventHandler oncopy;
          attribute EventHandler oncut;
          attribute EventHandler onpaste;
};
