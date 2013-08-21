/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-applet-element
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#the-applet-element
[NeedNewResolve]
interface HTMLAppletElement : HTMLElement {
    [Pure, SetterThrows]
        attribute DOMString align;
    [Pure, SetterThrows]
        attribute DOMString alt;
    [Pure, SetterThrows]
        attribute DOMString archive;
    [Pure, SetterThrows]
        attribute DOMString code;
    [Pure, SetterThrows]
        attribute DOMString codeBase;
    [Pure, SetterThrows]
        attribute DOMString height;
    [Pure, SetterThrows]
        attribute unsigned long hspace;
    [Pure, SetterThrows]
        attribute DOMString name;
    [Pure, SetterThrows]
        attribute DOMString _object;
    [Pure, SetterThrows]
        attribute unsigned long vspace;
    [Pure, SetterThrows]
        attribute DOMString width;
};

//HTMLAppletElement implements MozImageLoadingContent;
//HTMLAppletElement implements MozFrameLoaderOwner;
//HTMLAppletElement implements MozObjectLoadingContent;
