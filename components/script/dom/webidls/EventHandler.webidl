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

// https://html.spec.whatwg.org/multipage/#globaleventhandlers
[NoInterfaceObject]
interface GlobalEventHandlers {
           attribute EventHandler onabort;
           attribute EventHandler onblur;
           attribute EventHandler oncanplay;
           attribute EventHandler oncanplaythrough;
           attribute EventHandler onchange;
           attribute EventHandler onclick;
           attribute EventHandler ondblclick;
           attribute EventHandler onemptied;
           attribute OnErrorEventHandler onerror;
           attribute EventHandler oninput;
           attribute EventHandler onkeydown;
           attribute EventHandler onkeypress;
           attribute EventHandler onkeyup;
           attribute EventHandler onload;
           attribute EventHandler onloadeddata;
           attribute EventHandler onloadedmetadata;
           attribute EventHandler onmouseover;
           attribute EventHandler onpause;
           attribute EventHandler onplay;
           attribute EventHandler onplaying;
           attribute EventHandler onprogress;
           attribute EventHandler onreset;
           attribute EventHandler onresize;
           attribute EventHandler onsubmit;
           attribute EventHandler onsuspend;
           attribute EventHandler ontimeupdate;
           attribute EventHandler ontoggle;
           attribute EventHandler onwaiting;
};

// https://html.spec.whatwg.org/multipage/#windoweventhandlers
[NoInterfaceObject]
interface WindowEventHandlers {
           attribute EventHandler onunload;
           attribute EventHandler onstorage;
};
