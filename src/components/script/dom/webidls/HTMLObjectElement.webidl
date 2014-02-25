/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-object-element
 * http://www.whatwg.org/specs/web-apps/current-work/#HTMLObjectElement-partial
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#the-object-element
[NeedNewResolve]
interface HTMLObjectElement : HTMLElement {
  [Pure, SetterThrows]
           attribute DOMString data;
  [Pure, SetterThrows]
           attribute DOMString type;
  [Pure, SetterThrows]
           attribute DOMString name;
  [Pure, SetterThrows]
           attribute DOMString useMap;
  [Pure]
  readonly attribute HTMLFormElement? form;
  [Pure, SetterThrows]
           attribute DOMString width;
  [Pure, SetterThrows]
           attribute DOMString height;
  // Not pure: can trigger about:blank instantiation
  readonly attribute Document? contentDocument;
  // Not pure: can trigger about:blank instantiation
  readonly attribute WindowProxy? contentWindow;

  readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  void setCustomValidity(DOMString error);
};

// http://www.whatwg.org/specs/web-apps/current-work/#HTMLObjectElement-partial
partial interface HTMLObjectElement {
  [Pure, SetterThrows]
           attribute DOMString align;
  [Pure, SetterThrows]
           attribute DOMString archive;
  [Pure, SetterThrows]
           attribute DOMString code;
  [Pure, SetterThrows]
           attribute boolean declare;
  [Pure, SetterThrows]
           attribute unsigned long hspace;
  [Pure, SetterThrows]
           attribute DOMString standby;
  [Pure, SetterThrows]
           attribute unsigned long vspace;
  [Pure, SetterThrows]
           attribute DOMString codeBase;
  [Pure, SetterThrows]
           attribute DOMString codeType;

  [TreatNullAs=EmptyString, Pure, SetterThrows]
           attribute DOMString border;
};

partial interface HTMLObjectElement {
  // GetSVGDocument
  Document? getSVGDocument();
};
