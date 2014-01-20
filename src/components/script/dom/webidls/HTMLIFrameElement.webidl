/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-iframe-element
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

interface HTMLIFrameElement : HTMLElement {
  [SetterThrows, Pure]
           attribute DOMString src;
  [SetterThrows, Pure]
           attribute DOMString srcdoc;
  [SetterThrows, Pure]
           attribute DOMString name;
  // [PutForwards=value] readonly attribute DOMSettableTokenList sandbox;
  // We're implementing sandbox as a string for now, see bug 845057.
  attribute DOMString sandbox;
           // attribute boolean seamless;
  [SetterThrows, Pure]
           attribute boolean allowFullscreen;
  [SetterThrows, Pure]
           attribute DOMString width;
  [SetterThrows, Pure]
           attribute DOMString height;
  readonly attribute Document? contentDocument;
  readonly attribute WindowProxy? contentWindow;
};

// http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
partial interface HTMLIFrameElement {
  [SetterThrows, Pure]
           attribute DOMString align;
  [SetterThrows, Pure]
           attribute DOMString scrolling;
  [SetterThrows, Pure]
           attribute DOMString frameBorder;
  [SetterThrows, Pure]
           attribute DOMString longDesc;

  [TreatNullAs=EmptyString,SetterThrows,Pure] attribute DOMString marginHeight;
  [TreatNullAs=EmptyString,SetterThrows,Pure] attribute DOMString marginWidth;
};

partial interface HTMLIFrameElement {
  // GetSVGDocument
  Document? getSVGDocument();
};

/*partial interface HTMLIFrameElement {
  // nsIDOMMozBrowserFrame
  [ChromeOnly,SetterThrows]
           attribute boolean mozbrowser;
};

partial interface HTMLIFrameElement {
  // nsIMozBrowserFrame
  [ChromeOnly]
  readonly attribute DOMString appManifestURL;
};

HTMLIFrameElement implements MozFrameLoaderOwner;*/
