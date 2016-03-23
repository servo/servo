/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom/#the-stylesheet-interface
interface StyleSheet {
  readonly attribute DOMString type_;
  readonly attribute DOMString? href;

  // readonly attribute (Element or ProcessingInstruction)? ownerNode;
  // readonly attribute StyleSheet? parentStyleSheet;
  readonly attribute DOMString? title;

  // [SameObject, PutForwards=mediaText] readonly attribute MediaList media;
  // attribute boolean disabled;
};
