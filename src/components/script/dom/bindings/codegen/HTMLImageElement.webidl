/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#htmlimageelement
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

[NamedConstructor=Image(optional unsigned long width, optional unsigned long height)]
interface HTMLImageElement : HTMLElement {
           [SetterThrows]
           attribute DOMString alt;
           [SetterThrows]
           attribute DOMString src;
//           attribute DOMString srcset;
           [SetterThrows]
           attribute DOMString crossOrigin;
           [SetterThrows]
           attribute DOMString useMap;
           [SetterThrows]
           attribute boolean isMap;
           [SetterThrows]
           attribute unsigned long width;
           [SetterThrows]
           attribute unsigned long height;
  readonly attribute unsigned long naturalWidth;
  readonly attribute unsigned long naturalHeight;
  readonly attribute boolean complete;
};

// http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
partial interface HTMLImageElement {
           [SetterThrows]
           attribute DOMString name;
           [SetterThrows]
           attribute DOMString align;
           [SetterThrows]
           attribute unsigned long hspace;
           [SetterThrows]
           attribute unsigned long vspace;
           [SetterThrows]
           attribute DOMString longDesc;

  [TreatNullAs=EmptyString,SetterThrows] attribute DOMString border;
};
