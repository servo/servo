/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-area-element
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#the-area-element
interface HTMLAreaElement : HTMLElement {
    [SetterThrows]
        attribute DOMString alt;
    [SetterThrows]
        attribute DOMString coords;
    [SetterThrows]
        attribute DOMString shape;
    // No support for stringifier attributes yet
    //[SetterThrows]
    //stringifier attribute DOMString href;
//    stringifier;
    [SetterThrows]
        attribute DOMString href;
    [SetterThrows]
        attribute DOMString target;
    [SetterThrows]
        attribute DOMString download;
    [SetterThrows]
        attribute DOMString ping;

    // not implemented.
    //        [SetterThrows]
    //       attribute DOMString rel;
    //readonly attribute DOMTokenList relList;
    //
    //       [SetterThrows]
    //       attribute DOMString hreflang;
    //       [SetterThrows]
    //       attribute DOMString type;
};
//HTMLAreaElement implements URLUtils;

// http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
partial interface HTMLAreaElement {
    [SetterThrows]
        attribute boolean noHref;
};
