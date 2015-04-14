/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.whatwg.org/html/#htmlmetaelement
interface HTMLMetaElement : HTMLElement {
             attribute DOMString name;
  //         attribute DOMString httpEquiv;
             attribute DOMString content;

  // also has obsolete members
};

// https://www.whatwg.org/html/#HTMLMetaElement-partial
partial interface HTMLMetaElement {
  //         attribute DOMString scheme;
};
