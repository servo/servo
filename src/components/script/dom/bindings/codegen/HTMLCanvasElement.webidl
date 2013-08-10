/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-canvas-element
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// import from http://mxr.mozilla.org/mozilla-central/source/dom/webidl/HTMLCanvasElement.webidl

/*
interface Blob;
interface FileCallback;
interface nsIInputStreamCallback;
interface nsISupports;
interface PrintCallback;
interface Variant;
*/

interface HTMLCanvasElement : HTMLElement {
  [Pure, SetterThrows]
           attribute unsigned long width;
  [Pure, SetterThrows]
           attribute unsigned long height;
/*
  [Throws]
  nsISupports? getContext(DOMString contextId, optional any contextOptions = null);

  [Throws]
  DOMString toDataURL(optional DOMString type = "",
                      optional any encoderOptions);
  [Throws]
  void toBlob(FileCallback _callback,
              optional DOMString type = "",
              optional any encoderOptions);
*/
};
/*
// Mozilla specific bits
partial interface HTMLCanvasElement {
  [Pure, SetterThrows]
           attribute boolean mozOpaque;
  [Throws]
  File mozGetAsFile(DOMString name, optional DOMString? type = null);
  [ChromeOnly, Throws]
  nsISupports? MozGetIPCContext(DOMString contextId);
  [ChromeOnly]
  void mozFetchAsStream(nsIInputStreamCallback callback, optional DOMString? type = null);
           attribute PrintCallback? mozPrintCallback;
};
*/
